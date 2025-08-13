// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct BitSet<const N: usize>
// where
//     [(); (N + 63) / 64]:,
// {
//     // Each u64 stores 64 bits
//     data: [u64; (N + 63) / 64],
// }
//
// impl<T: LimitedElementSet, const N: usize> SetTraits<T> for BitSet<N>
// where
//     [(); (N + 63) / 64]:,
// {
//     /// Create an empty BitSet
//     pub fn new() -> Self {
//         Self {
//             data: [0; (N + 63) / 64],
//         }
//     }
//
//     /// Insert an element
//     pub fn insert(&mut self, value: usize) -> bool {
//         assert!(value < N, "BitSet index out of bounds");
//         let (block, bit) = (value / 64, value % 64);
//         let mask = 1u64 << bit;
//         let already_present = self.data[block] & mask != 0;
//         self.data[block] |= mask;
//         !already_present
//     }
//
//     /// Remove an element
//     pub fn remove(&mut self, value: usize) -> bool {
//         assert!(value < N, "BitSet index out of bounds");
//         let (block, bit) = (value / 64, value % 64);
//         let mask = 1u64 << bit;
//         let was_present = self.data[block] & mask != 0;
//         self.data[block] &= !mask;
//         was_present
//     }
//
//     /// Check if the element exists
//     pub fn contains(&self, value: usize) -> bool {
//         assert!(value < N, "BitSet index out of bounds");
//         let (block, bit) = (value / 64, value % 64);
//         (self.data[block] >> bit) & 1 != 0
//     }
//
//     /// Clear the set
//     pub fn clear(&mut self) {
//         for word in &mut self.data {
//             *word = 0;
//         }
//     }
//
//     /// Check if the set is empty
//     pub fn is_empty(&self) -> bool {
//         self.data.iter().all(|&x| x == 0)
//     }
//
//     /// Count how many bits are set
//     pub fn len(&self) -> usize {
//         self.data.iter().map(|x| x.count_ones() as usize).sum()
//     }
//
//     /// Iterate over the set bits
//     pub fn iter(&self) -> BitSetIter<N> {
//         BitSetIter {
//             bitset: self,
//             current: 0,
//         }
//     }
// }
//
// /// Iterator over the BitSet
// pub struct BitSetIter<'a, const N: usize>
// where
//     [(); (N + 63) / 64]:,
// {
//     bitset: &'a BitSet<N>,
//     current: usize,
// }
//
// impl<'a, const N: usize> Iterator for BitSetIter<'a, N>
// where
//     [(); (N + 63) / 64]:,
// {
//     type Item = usize;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.current < N {
//             let val = self.current;
//             self.current += 1;
//             if self.bitset.contains(val) {
//                 return Some(val);
//             }
//         }
//         None
//     }
// }
//
