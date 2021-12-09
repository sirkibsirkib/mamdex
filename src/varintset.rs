use core::ops::Range;

pub struct FmIndex {
    f_l: Box<[[u8; 2]]>,
}

pub struct FmQuery<'a> {
    f_l: &'a [[u8; 2]],
    range: Range<usize>,
}
impl FmQuery<'_> {
    fn add_to_prefix(&mut self, byte: u8) {}
}

impl FmIndex {
    pub fn query(&self) -> FmQuery {
        FmQuery { f_l: &self.f_l, range: 0..self.f_l.len() }
    }
    pub fn new(data: &[u8]) -> Self {
        assert!(data.iter().all(|&x| x != 0x00));

        let mut pointers: Box<[*const u8]> =
            (0..data.len()).map(|offset| unsafe { data.as_ptr().add(offset) }).collect();

        let end = unsafe { data.as_ptr().add(data.len()) };
        pointers.sort_by_key(|&ptr| unsafe {
            std::slice::from_raw_parts(ptr, end as usize - ptr as usize)
        });

        let rest = pointers.iter().map(|&ptr| unsafe {
            let f = *ptr;
            let l = if ptr == data.as_ptr() { 0x00 } else { *ptr.sub(1) };
            [f, l]
        });
        let me = Self {
            f_l: std::iter::once([0, data.last().copied().unwrap_or(0)]).chain(rest).collect(),
        };

        for &[f, l] in me.f_l.iter() {
            println!("{:?}", [f as char, l as char]);
        }
        me
    }
}
