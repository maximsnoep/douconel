use std::ops::Add;

pub fn average<'a, T>(list: impl Iterator<Item = T>) -> T
where
    T: Add<Output = T> + std::default::Default + std::ops::Div<f32, Output = T>,
{
    let (sum, count) = list.fold((T::default(), 0.), |(sum, count), elem| {
        (sum + elem, count + 1.)
    });
    sum / (count as f32)
}
