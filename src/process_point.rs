use rten_tensor::NdTensor;

use crate::Point;

fn process(points: Vec<Point>) ->  NdTensor<f32, 3> {
    
    
    NdTensor::zeros([1, 1, 1])
}

pub fn to_matrix(points: &Vec<Point>) -> Vec<Box<[f64; 1000]>>{

    let min_x = points.iter().min_by_key(|p| p.x as i32).unwrap().x;
    let min_y = points.iter().min_by_key(|p| p.y as i32).unwrap().y;
    let max_x = points.iter().max_by_key(|p| p.x as i32).unwrap().x;
    let max_y = points.iter().max_by_key(|p| p.y as i32).unwrap().y;


    let x_len = max_x - min_x;
    let y_len = max_y - min_y;
    let x_scale = 800.0 / x_len;
    let y_scale = (y_len / x_len) * x_scale;

    let scaled_points = points.iter().map(|point| {
        let x_scaled = ((point.x - min_x) * x_scale) + 100.0;
        let y_scaled = ((point.y - min_y) * x_scale) + (100.0 * y_scale);

        ((x_scaled, y_scaled), point.new_line)
    });

    let mut matrix: Vec<Box<[f64; 1000]>> = vec![Box::new([-0.0; 1000]); (y_len * y_scale) as u32 as usize];

    for ((x, y), newline) in scaled_points {
        let start_x = x - 1.5;
        let end_x = x + 1.5;
        let start_y = y - 1.5;
        let end_y = y + 1.5;

        matrix.iter_mut().enumerate().for_each(|(y, line)| line.iter_mut().for_each(|x| {
            if (start_x < *x && *x < end_x) && (start_y < (y as f64)  && (y as f64) < end_y) {
                *x = 1.0;
            }
        }));
    }



    matrix

}
