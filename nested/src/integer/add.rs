use {
    std::sync::{Arc, RwLock},
    crate::{
        core::{InnerViewPort, OuterViewPort},
        sequence::SequenceView,
        vec::VecBuffer,
        projection::ProjectionHelper
    }
};

fn posint_add(
    radix: usize,
    a: impl SequenceView<Item = usize>,
    b: impl SequenceView<Item = usize>
) -> Vec<usize> {
    let mut carry = 0;
    let mut result = Vec::new();

    for digit_idx in 0 .. std::cmp::max(a.len().unwrap_or(0), b.len().unwrap_or(0)) {
        let sum =
            a.get(&digit_idx).unwrap_or(0) +
            b.get(&digit_idx).unwrap_or(0) +
            carry;

        result.push(sum % radix);

        carry =
            if sum > radix {
                sum - radix
            } else {
                0
            };
    }

    if carry > 0 {
        result.push(carry);
    }

    result
}

pub struct Add {
    radix: usize,
    a: Arc<dyn SequenceView<Item = usize>>, // PosInt, Little Endian
    b: Arc<dyn SequenceView<Item = usize>>, // PosInt, Little Endian
    c: VecBuffer<usize>,
    _proj_helper: ProjectionHelper<Self>
}

impl Add {
    pub fn new(
        radix: usize,
        a: OuterViewPort<dyn SequenceView<Item = usize>>,
        b: OuterViewPort<dyn SequenceView<Item = usize>>,
        c: InnerViewPort<RwLock<Vec<usize>>>//<dyn SequenceView<Item = usize>>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(c.0.update_hooks.clone());
        let add = Arc::new(RwLock::new(
            Add {
                radix,
                a: proj_helper.new_sequence_arg(a, |s: &mut Self, _digit_idx| s.update()),
                b: proj_helper.new_sequence_arg(b, |s: &mut Self, _digit_idx| s.update()),
                c: VecBuffer::new(c),
                _proj_helper: proj_helper
            }
        ));
        add
    }

    fn update(&mut self) {
        self.c.clear();
        for digit in posint_add(self.radix, self.a.clone(), self.b.clone()) {
            self.c.push(digit);
        }
    }
}
