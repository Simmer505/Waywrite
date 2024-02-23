use std::fs;
use std::error::Error;
use std::time::Instant;

use rten_tensor::{NdTensor, AsView};
use rten::Model;
use ocrs::{OcrEngine, OcrEngineParams};

use crate::Point;

const MATRIX_SIZE: usize = 800;

pub fn print_written(points: &Vec<Point>) -> Result<(), Box<dyn Error>> {

    let begin = Instant::now();
    let processed_data = process(points);

    println!("{:#?}", begin.elapsed());
    let begin = Instant::now();
    ocr(processed_data)?;
    println!("{:#?}", begin.elapsed());

    Ok(())
}

fn process(points: &Vec<Point>) ->  NdTensor<f32, 3> {

    let matrix = to_matrix(points);
    let y_len = matrix[0].len();
    let x_len = matrix[0][0].len();
    let data: Vec<f32> = matrix.into_iter().flatten().flatten().map(|f| f as f32).collect();
    
    
    NdTensor::from_data([1, y_len, x_len], data)
}

fn ocr(data: NdTensor<f32, 3>) -> Result<(), Box<dyn Error>> {

    let detection_model_data = fs::read("text-detection.rten")?;
    let rec_model_data = fs::read("text-recognition.rten")?;

    let detection_model = Model::load(&detection_model_data)?;
    let rec_model = Model::load(&rec_model_data)?;


    let ocr_engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(rec_model),
        ..Default::default()
    })?;

    let input = ocr_engine.prepare_input(data.view())?;

    let word_rects = ocr_engine.detect_words(&input)?;

    let line_rects = ocr_engine.find_text_lines(&input, &word_rects);

    let line_texts = ocr_engine.recognize_text(&input, &line_rects)?;

    for line in line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1) 
    {
        println!("{}", line);
    }

    

    Ok(())

    
}

fn line(x: f64, point1: (f64, f64), point2: (f64, f64)) -> f64 {
    let slope = (point2.1 - point1.1) / (point2.0 - point1.0);

    let point = slope * (x - point1.0) + point1.1;

    point
}

fn to_matrix(points: &Vec<Point>) -> Vec<Vec<Vec<f64>>> {

    let min_x = points.iter().min_by_key(|p| p.x as i32).unwrap().x;
    let min_y = points.iter().min_by_key(|p| p.y as i32).unwrap().y;
    let max_x = points.iter().max_by_key(|p| p.x as i32).unwrap().x;
    let max_y = points.iter().max_by_key(|p| p.y as i32).unwrap().y;

    let x_len = max_x - min_x;
    let y_len = max_y - min_y;

    let y_ratio = y_len / x_len;

    let x_size = MATRIX_SIZE as f64 * 0.5;
    let y_size = (MATRIX_SIZE as f64 * 0.5) * y_ratio;

    let x_offset = (MATRIX_SIZE as f64 - x_size) / 2.0;
    let y_offset = ((MATRIX_SIZE as f64 * y_ratio) - y_size) / 2.0;

    let x_scale = x_size / x_len;
    let y_scale = y_size / y_len;

    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; MATRIX_SIZE]; (MATRIX_SIZE as f64 * y_ratio) as usize];


    let scaled_points = points
        .iter()
        .map(|point| {
            let x_scaled = ((point.x - min_x) * x_scale) + x_offset;
            let y_scaled = ((point.y - min_y) * y_scale) + y_offset;

            ((x_scaled, y_scaled), point.new_line)
    }).collect::<Vec<_>>();

    let line_width_x = MATRIX_SIZE as f64 / 80.0;
    let line_width_y = (MATRIX_SIZE as f64 * y_ratio) / 80.0;

    let mut last_x = 0.0;
    let mut last_y = 0.0;

    for ((x, y), newline) in scaled_points {

        if !newline {
            let curr_x_start = x - (line_width_x / 2.0);
            let curr_x_end = x + (line_width_x / 2.0);

            let last_x_start = last_x - (line_width_x / 2.0);
            let last_x_end = last_x + (line_width_x / 2.0);

            let top_y: f64;
            let bottom_y: f64;

            if y > last_y {
                top_y = y + ((line_width_y / 2.0) * y_scale);
                bottom_y = last_y - ((line_width_y / 2.0) * y_scale);
            } else {
                top_y = last_y + ((line_width_y / 2.0) * y_scale);
                bottom_y = y - ((line_width_y / 2.0) * y_scale);
            }

            let start_x = (last_x_start.min(curr_x_start)) as usize;
            let end_x = (last_x_end.max(curr_x_end)) as usize + 1;


            for x in start_x..(end_x + 1) {

                let left_line_y = line(x as f64, (last_x_start, last_y), (curr_x_start, y));
                let right_line_y = line(x as f64, (last_x_end, last_y), (curr_x_end, y));

                let top_line = left_line_y.max(right_line_y);
                let bottom_line = left_line_y.min(right_line_y);

                let top_line = top_line.min(top_y) as usize + 1;
                let bottom_line = bottom_line.max(bottom_y) as usize;

                for line_y in bottom_line..(top_line + 1) {
                    matrix[line_y][x] = 1.0;
                }

            }
        }

        last_x = x;
        last_y = y;

    }

    vec![matrix]

}
