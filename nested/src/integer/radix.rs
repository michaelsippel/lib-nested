use {
    std::sync::{Arc, RwLock},
    crate::{
        core::{
            Observer,
            InnerViewPort,
            OuterViewPort
        },
        sequence::SequenceView,
        vec::VecBuffer
    }
};

pub struct RadixProjection {
    src_radix: usize,
    dst_radix: usize,
    src_digits: Option<Arc<dyn SequenceView<Item = usize>>>,
    dst_digits: RwLock<VecBuffer<usize>>
}

impl RadixProjection {
    pub fn new(
        src_radix: usize,
        dst_radix: usize,
        src_digits: OuterViewPort<dyn SequenceView<Item = usize>>,
        dst_digits: InnerViewPort<RwLock<Vec<usize>>>
    ) -> Arc<RwLock<Self>> {
        dst_digits.0.add_update_hook(Arc::new(src_digits.0.clone()));
        let proj = Arc::new(RwLock::new(
            RadixProjection {
                src_radix,
                dst_radix,
                src_digits: None,
                dst_digits: RwLock::new(VecBuffer::new(dst_digits))
            }
        ));
        src_digits.add_observer(proj.clone());
        proj
    }

    fn machine_int(&self) -> usize {
        let mut val = 0;
        let mut r = 1;
        for i in 0 .. self.src_digits.len().unwrap_or(0) {
            val += r * self.src_digits.get(&i).unwrap();
            r *= self.src_radix;
        }

        val
    }

    // recalculate everything
    fn update(&self) {
        let mut dst = self.dst_digits.write().unwrap();
        dst.clear();

        let mut val = self.machine_int();

        while val > 0 {
            dst.push(val % self.dst_radix);
            val /= self.dst_radix;
        }        
    }

    fn _update_dst_digit(&mut self, _idx: usize) {
        /*
        let v = 0; // calculate new digit value

        // which src-digits are responsible?

        if idx < self.dst_digits.len() {
            self.dst_digits.get_mut(idx) = v;
        } else if idx == self.dst_digits.len() {
            self.dst_digits.push(v);
        } else {
            // error
        }
*/
    }
}

impl Observer<dyn SequenceView<Item = usize>> for RadixProjection {
    fn reset(&mut self, view: Option<Arc<dyn SequenceView<Item = usize>>>) {
        self.src_digits = view;
    }

    fn notify(&mut self, _idx: &usize) {
        // todo:
        // src digit i changed.
        // which dst-digits does it affect?
        // update dst-digit j:

        // ...but for now the easy way
        self.update();
    }
}
