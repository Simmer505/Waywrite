use rten_tensor::NdTensor;

use crate::Point;

fn process(points: Vec<Point>) ->  NdTensor<f32, 3> {
    
    
    NdTensor::zeros([1, 1, 1])
}

pub fn to_matrix(points: &Vec<Point>) -> Vec<Box<[f64; 200]>>{

    const MATRIX_SIZE: f64 = 200.0;

    let min_x = points.iter().min_by_key(|p| p.x as i32).unwrap().x;
    let min_y = points.iter().min_by_key(|p| p.y as i32).unwrap().y;
    let max_x = points.iter().max_by_key(|p| p.x as i32).unwrap().x;
    let max_y = points.iter().max_by_key(|p| p.y as i32).unwrap().y;


    let x_len = max_x - min_x;
    let y_len = max_y - min_y;
    let y_ratio = y_len / x_len;
    let x_scale = (0.8 * MATRIX_SIZE) / x_len;
    let y_scale = ((0.8 * MATRIX_SIZE) * y_ratio) / y_len;

    let scaled_points = points.iter().map(|point| {
        let x_scaled = ((point.x - min_x) * x_scale) + (0.1 * MATRIX_SIZE);
        let y_scaled = ((point.y - min_y) * y_scale) + ((0.1 * MATRIX_SIZE) * y_ratio);

        ((x_scaled, y_scaled), point.new_line)
    });

    let mut matrix: Vec<Box<[f64; MATRIX_SIZE as usize]>> = vec![Box::new([0.0; MATRIX_SIZE as u32 as usize]); (MATRIX_SIZE * y_ratio) as u64 as usize];

    for ((x, y), newline) in scaled_points {
        let start_x = x - (MATRIX_SIZE / 100.0);
        let end_x = x + (MATRIX_SIZE / 100.0);
        let start_y = y - ((MATRIX_SIZE / 100.0) * y_ratio);
        let end_y = y + ((MATRIX_SIZE / 100.0) * y_ratio);

        matrix.iter_mut().enumerate().for_each(|(mat_y, line)| line.iter_mut().enumerate().for_each(|(mat_x, val)| {
            if (start_x < (mat_x as f64) && (mat_x as f64) < end_x) && (start_y < (mat_y as f64)  && (mat_y as f64) < end_y) {
                *val = 1.0;
            }
        }));
    }



    matrix

}
