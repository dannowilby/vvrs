
#[derive(Debug, Clone, Copy)]
struct Space {
    offset: u64,
    length: u64,
}

impl Space {
    fn new(offset: u64, length: u64) -> Self {
        Self {
            offset,
            length
        }
    }
}

/// An allocator with fixed size that abstracts over a block of memory. Used for
/// managing the allocation of our vertex buffer.
#[derive(Default)]
pub struct Allocator {
    size: u64,
    freelist: Vec<Space>,
    occulist: Vec<Space>,
}

impl Allocator {

    pub fn new(size: u64) -> Self {
        Self {
            size,
            freelist: vec![Space::new(0, size)],
            occulist: vec![]
        }
    }

    pub fn alloc(&mut self, length: u64) -> Option<u64> {
        
        let mut min: Option<usize> = None;
        let mut min_diff = 0;

        for i in 0..self.freelist.len() {

            let diff = self.freelist[i].length as i64 - length as i64;

            if diff < 0 {
                continue;
            }

            if min.is_none() {
                min = Some(i);
                min_diff = diff;
                continue;
            }

            if diff < min_diff {
                min = Some(i);
                min_diff = diff;
            }
            
        }
        let i = min?;
        
        let fs = self.freelist.remove(i);
        let offset = fs.offset;
        
        let new_freespace = Space {
            offset: offset + length,
            length: fs.length - length
        };
        if new_freespace.length != 0 {
            self.freelist.push(new_freespace);
        }
        
        let occuspace = Space {
            offset,
            length
        };
        self.occulist.push(occuspace);

        Some(offset)
    }

    pub fn dealloc(&mut self, offset: u64) {

        let i = self.occulist.iter().position(|x| x.offset == offset).expect("");

        let space = self.occulist.remove(i);

        // check if can merge with a space already in the list
        let mut distinct = true;
        for f in self.freelist.iter_mut() {
            
            if f.offset + f.length == space.offset {
                f.length += space.length;
                distinct = false;
                break;
            }

            if space.offset + space.length == f.offset {
                f.offset -= space.length;
                distinct = false;
                break;
            }

        }

        // if can't merge, add it
        if distinct {
            self.freelist.push(space);
            return;            
        }

        // if merged, check if it's a double merge
        for i in 0..self.freelist.len() {
            for j in 0..self.freelist.len() {
                
                if i == j { continue; }

                let space_i = self.freelist[i];
                let space_j = self.freelist[j];

                if space_i.offset == space_j.offset + space_j.length {
                    self.freelist[j].length += space_i.length;
                    self.freelist.remove(i);
                    return;
                }
            }
        }

    }

    pub fn percent_full(&self) -> f32 {

        let mut allocked = 0;

        for i in &self.occulist {
            allocked += i.length;
        }

        allocked as f32 / self.size as f32
    }

}

#[cfg(test)]
mod test {
    use super::Allocator;

    #[test]
    fn gpu_allocs_in_order() {
        let mut x = Allocator::new(1024);

        let offset = x.alloc(24).expect("");
        assert!(offset == 0);

        let offset = x.alloc(10).expect("");
        assert!(offset == 24);

        let offset = x.alloc(11).expect("");
        assert!(offset == 34);
    }

    #[test]
    fn gpu_allocs_fit_blocks() {

        let mut x = Allocator::new(1024);

        let _ = x.alloc(24).expect("");
        let offset = x.alloc(10).expect("");
        let _ = x.alloc(11).expect("");
        
        x.dealloc(offset);

        let offset = x.alloc(10).expect("");
        assert!(offset == 24);
    }

    #[test]
    fn allocs_merge_freespace() {
        let mut x = Allocator::new(1024);

        let offset1 = x.alloc(24).expect("");
        let offset2 = x.alloc(10).expect("");
        let offset3 = x.alloc(11).expect("");
        let _ = x.alloc(12).expect("");

        x.dealloc(offset1);
        x.dealloc(offset3);
        x.dealloc(offset2);

        let t = x.alloc(45).expect("");
        assert!(t == 0);
    }

    #[test]
    fn big_allocs_dont_find_space() {
        let mut x = Allocator::new(1024);
        let t = x.alloc(2048);
        assert!(t.is_none());
    }

    #[test]
    fn percent_allocked() {
        let mut x = Allocator::new(2048);
        let _ = x.alloc(1024);
        let percent = x.percent_full();
        assert!(percent == 0.5);
    }
}