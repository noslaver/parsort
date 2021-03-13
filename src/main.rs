use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

fn main() {
    let opts: Vec<_> = std::env::args().collect();

    let cores = opts[1].parse::<usize>().unwrap();
    let input = &opts[2];

    let mut nums = read_numbers(&input).unwrap();

    let start = Instant::now();

    // sort
    parallel_sort(&mut nums, cores);

    let duration = Instant::now() - start;
    println!("MergeSort: {}", duration.as_micros());

    for i in nums {
        println!("{}", i);
    }
}

fn read_numbers<P: AsRef<Path>>(path: P) -> io::Result<Vec<usize>> {
    let nums = BufReader::new(File::open(path)?)
        .lines()
        .map(|l| l.unwrap().parse::<usize>().unwrap())
        .collect::<Vec<_>>();

    Ok(nums)
}

fn merge(arr1: &[usize], arr2: &[usize], ret: &mut [usize]) {
    let mut left = 0; // Head of left pile
    let mut right = 0; // Head of right pile
    let mut index = 0;

    // compare element and insert back to result array
    while left < arr1.len() && right < arr2.len() {
        if arr1[left] <= arr2[right] {
            ret[index] = arr1[left];
            index += 1;
            left += 1;
        } else {
            ret[index] = arr2[right];
            index += 1;
            right += 1;
        }
    }

    // copy the reset elements to returned array,
    if left < arr1.len() {
        ret[index..].copy_from_slice(&arr1[left..]);
    }
    if right < arr2.len() {
        ret[index..].copy_from_slice(&arr2[right..]);
    }
}

fn mergesort(buff: &mut [usize]) {
    let len = buff.len();
    if len == 1 {
        return;
    }
    let partition = len / 2;
    let (left, right) = buff.split_at_mut(partition);

    mergesort(left);
    mergesort(right);

    let mut sorted = vec![0usize; len];

    merge(left, right, &mut sorted[..]);

    buff.copy_from_slice(&sorted[..]);
}

#[derive(Debug, Eq)]
struct Item<'a> {
    arr: &'a [usize],
    idx: usize,
}

impl<'a> Item<'a> {
    fn new(arr: &'a [usize], idx: usize) -> Self {
        Self { arr, idx }
    }

    fn item(&self) -> usize {
        self.arr[self.idx]
    }
}

impl<'a> PartialEq for Item<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.item() == other.item()
    }
}

impl<'a> PartialOrd for Item<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.item().partial_cmp(&other.item())
    }
}

impl<'a> Ord for Item<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item().cmp(&other.item())
    }
}

fn merge_k_sorted(arrays: &[&[usize]]) -> Vec<usize> {
    let k = arrays.len();

    let mut sorted = vec![];

    let mut heap = BinaryHeap::with_capacity(k);

    for arr in arrays {
        let item = Item::new(arr, 0);
        heap.push(std::cmp::Reverse(item));
    }

    while !heap.is_empty() {
        let mut item = heap.pop().unwrap();
        sorted.push(item.0.item());
        item.0.idx += 1;
        if item.0.idx < item.0.arr.len() {
            heap.push(item);
        }
    }

    sorted
}

fn parallel_sort(input_vec: &mut [usize], num_threads: usize) {
    let len = input_vec.len();

    let (tx, rx) = mpsc::channel();

    for i in 0..num_threads {
        let range = (len * i / num_threads)..(len * (i + 1) / num_threads);
        let chunk = &mut input_vec[range];
        let chunk_len = chunk.len();
        let chunk = chunk.as_mut_ptr() as usize;
        let tx = tx.clone();
        thread::spawn(move || {
            let mut chunk = unsafe {
                let chunk = chunk as *mut usize;
                std::slice::from_raw_parts_mut(chunk, chunk_len)
            };
            mergesort(&mut chunk);
            let chunk = chunk.as_mut_ptr() as usize;
            tx.send((chunk, chunk_len)).unwrap();
            drop(tx); // explicit
        });
    }

    drop(tx);

    // use current thread as receiver
    let arrays = rx
        .iter()
        .map(|(chunk, len)| unsafe {
            let chunk = chunk as *mut usize;
            std::slice::from_raw_parts(chunk, len)
        })
        .collect::<Vec<_>>();

    let sorted = merge_k_sorted(&arrays);

    input_vec.copy_from_slice(&sorted[..]);
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use rand::Rng;
//
//    fn is_sorted(arr: &[usize]) -> bool {
//        arr.windows(2).all(|w| w[0] <= w[1])
//    }
//
//    #[test]
//    fn mergesort_works() {
//        let input = read_numbers("../input.txt").unwrap();
//        let output = read_numbers("../sorted.txt").unwrap();
//
//        let mut sorted = input.clone();
//        mergesort(&mut sorted);
//
//        assert_eq!(sorted, output);
//    }
//
//    #[test]
//    fn merge_k_sorted_works() {
//        let mut arr1 = [0usize; 2048];
//        let mut arr2 = [0usize; 2048];
//        let mut arr3 = [0usize; 2048];
//
//        rand::thread_rng().fill(&mut arr1);
//        rand::thread_rng().fill(&mut arr2);
//        rand::thread_rng().fill(&mut arr3);
//
//        arr1.sort();
//        arr2.sort();
//        arr3.sort();
//
//        let sorted = merge_k_sorted(&[&arr1, &arr2, &arr3]);
//
//        assert!(is_sorted(&sorted));
//    }
//
//    #[test]
//    fn it_works_par() {
//        let mut input = read_numbers("../input.txt").unwrap();
//
//        parallel_sort(&mut input, 4);
//
//        assert!(is_sorted(&input));
//    }
//}
