use {
    std::{
        sync::Arc,
        io::Write
    },
    std::sync::RwLock,
    async_std::{
        io::{Read, ReadExt},
        stream::{StreamExt}
    },
    serde::{Serialize, de::DeserializeOwned},
    crate::{
        core::{Observer, OuterViewPort},
        vec::{VecDiff, VecBuffer}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct VecJsonWriter<T, W>
where T: Clone + Send + Sync + 'static,
      W: Write + Send + Sync {
    data: Option<Arc<RwLock<Vec<T>>>>,
    out: RwLock<W>
}

impl<T> OuterViewPort<RwLock<Vec<T>>>
where T: Clone + Serialize + Send + Sync + 'static {
    pub fn serialize_json<W: Write + Send + Sync + 'static>(&self, out: W) -> Arc<RwLock<VecJsonWriter<T, W>>> {
        let writer = Arc::new(RwLock::new(
            VecJsonWriter {
                data: None,
                out: RwLock::new(out),
            }
        ));
        self.add_observer(writer.clone());
        writer
    }
}

impl<T, W> Observer<RwLock<Vec<T>>> for VecJsonWriter<T, W>
where T: Clone + Serialize + Send + Sync + 'static,
      W: Write + Send + Sync
{
    fn reset(&mut self, view: Option<Arc<RwLock<Vec<T>>>>) {
        self.data = view;

        self.out.write().unwrap().write(&serde_json::to_string(&VecDiff::<T>::Clear).unwrap().as_bytes()).expect("");
        self.out.write().unwrap().write(b"\n").expect("");

        if let Some(data) = self.data.as_ref() {
            for x in data.read().unwrap().iter() {
                self.out.write().unwrap().write(&serde_json::to_string(&VecDiff::Push(x)).unwrap().as_bytes()).expect("");
                self.out.write().unwrap().write(b"\n").expect("");
            }
        }

        self.out.write().unwrap().flush().expect("");
    }

    fn notify(&mut self, diff: &VecDiff<T>) {
        self.out.write().unwrap().write(serde_json::to_string(diff).unwrap().as_bytes()).expect("");
        self.out.write().unwrap().write(b"\n").expect("");
        self.out.write().unwrap().flush().expect("");
    }
}

impl<T> VecBuffer<T>
where T: DeserializeOwned + Clone + Send + Sync + 'static
{
    pub async fn from_json<R: Read + async_std::io::Read + Unpin>(&mut self, read: R) {
        let mut bytes = read.bytes();
        let mut s = String::new();
        while let Some(Ok(b)) = bytes.next().await {
            match b {
                b'\n' => {
                    if s.len() > 0 {
                        let diff = serde_json::from_str::<VecDiff<T>>(&s).expect("error parsing json");
                        self.apply_diff(diff);
                        s.clear();
                    }
                },
                c => {
                    s.push(c as char);
                }
            }
        }
    }
}


