pub trait FindContinuousEmptyStart<T> {
    fn find_continuous_empty_start(
        &mut self,
        empty_checker: Box<dyn Fn(&T) -> bool>,
        empty_generator: Box<dyn Fn() -> T>,
        length: usize,
    ) -> usize;
}

impl<T> FindContinuousEmptyStart<T> for Vec<T> {
    fn find_continuous_empty_start(
        self: &mut Vec<T>,
        empty_checker: Box<dyn Fn(&T) -> bool>,
        empty_generator: Box<dyn Fn() -> T>,
        length: usize,
    ) -> usize {
        let mut current_start: Option<usize> = None;

        #[allow(clippy::needless_range_loop)]
        // 遍历现有元素以寻找可能的起始位置
        for i in 0..self.len() {
            if empty_checker(&self[i]) {
                if current_start.is_none() {
                    current_start = Some(i);
                }
            } else if let Some(start) = current_start {
                // 检查当前区间是否满足长度要求
                if i - start >= length {
                    return start;
                }
                current_start = None;
            }
        }

        // 检查最后一个可能区间
        if let Some(start) = current_start {
            let available = self.len() - start;
            return if available >= length {
                start
            } else {
                // 扩展该区间以满足长度要求
                self.resize_with(start + length, empty_generator);
                start
            };
        }

        // 没有找到任何空位，直接在末尾扩展
        let index = self.len();
        self.resize_with(index + length, empty_generator);
        index
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;

    #[test]
    fn test_find_continuous_empty_start() {
        let mut v = vec![0, 100, 2903, 4390, 0, 0, 0, 0, 0, 0];
        let is_empty = Box::new(|i: &i32| *i == 0);
        let def = Box::new(i32::default);
        let d1 = dbg!(v.find_continuous_empty_start(is_empty.clone(), def.clone(), 1));
        let d2 = dbg!(v.find_continuous_empty_start(is_empty.clone(), def.clone(), 5));
        v.fill(10);
        let d3 = dbg!(v.find_continuous_empty_start(is_empty.clone(), def.clone(), 11));
        dbg!(v);
    }
}
