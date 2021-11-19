use {
    crate::{
        core::{Observer, OuterViewPort},
        vec::VecDiff,
    },
    serde::Serialize,
    std::sync::RwLock,
    std::{io::Write, sync::Arc},
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Serialization Observer for `Vec`
pub struct VecBinWriter<T, W>
where
    T: Clone + Send + Sync + 'static,
    W: Write + Send + Sync,
{
    data: Option<Arc<RwLock<Vec<T>>>>,
    out: RwLock<W>,
}

impl<T> OuterViewPort<RwLock<Vec<T>>>
where
    T: Clone + Serialize + Send + Sync + 'static,
{
    pub fn serialize_bin<W: Write + Send + Sync + 'static>(
        &self,
        out: W,
    ) -> Arc<RwLock<VecBinWriter<T, W>>> {
        let writer = Arc::new(RwLock::new(VecBinWriter {
            data: None,
            out: RwLock::new(out),
        }));
        self.add_observer(writer.clone());
        writer
    }
}

impl<T, W> Observer<RwLock<Vec<T>>> for VecBinWriter<T, W>
where
    T: Clone + Serialize + Send + Sync + 'static,
    W: Write + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<RwLock<Vec<T>>>>) {
        self.data = view;
        let mut out = self.out.write().unwrap();

        out.write(
            &bincode::serialized_size(&VecDiff::<T>::Clear)
                .unwrap()
                .to_le_bytes(),
        )
        .expect("");
        out.write(&bincode::serialize(&VecDiff::<T>::Clear).unwrap())
            .expect("");

        if let Some(data) = self.data.as_ref() {
            for x in data.read().unwrap().iter() {
                out.write(
                    &bincode::serialized_size(&VecDiff::Push(x))
                        .unwrap()
                        .to_le_bytes(),
                )
                .expect("");
                out.write(&bincode::serialize(&VecDiff::Push(x)).unwrap())
                    .expect("");
            }
        }

        out.flush().expect("");
    }

    fn notify(&mut self, diff: &VecDiff<T>) {
        let mut out = self.out.write().unwrap();
        out.write(&bincode::serialized_size(diff).unwrap().to_le_bytes())
            .expect("");
        out.write(&bincode::serialize(diff).unwrap()).expect("");
        out.flush().expect("");
    }
}
