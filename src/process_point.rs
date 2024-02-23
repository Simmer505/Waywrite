use std::fs;
use std::error::Error;
use std::time::Instant;

use rten_tensor::{NdTensor, AsView};
use rten::Model;
use ocrs::{OcrEngine, OcrEngineParams};
use image::{ColorType, ImageFormat};

use crate::Point;

const MATRIX_LEN: usize = 800;
const TEXT_MATRIX_RATIO: f64 = 0.5;
const LINE_WIDTH: f64 = 10.0;

const MATRIX_X_SIZE: f64 = MATRIX_LEN as f64;
const TEXT_X_SIZE: f64 = (MATRIX_X_SIZE as f64) * TEXT_MATRIX_RATIO;
const TEXT_X_OFFSET: f64 = (MATRIX_X_SIZE - TEXT_X_SIZE) / 2.0;
const LINE_WIDTH_X_OFFSET: f64 = LINE_WIDTH / 2.0;

pub fn print_words(points: &Vec<Point>) -> Result<(), Box<dyn Error>> {

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
    let image_data: Box<[u8]> = matrix.iter().flatten().flatten().map(|f| if *f > 0.5 { u8::from(0) } else { u8::from(255) }).collect();
    let data: Vec<f32> = matrix.into_iter().flatten().flatten().map(|f| f as f32).collect();

    image::save_buffer_with_format("./image.png", &image_data, x_len as u32, y_len as u32, ColorType::L8, ImageFormat::Png).unwrap();
    
    
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
    
    let matrix_y_size = MATRIX_X_SIZE * y_ratio;
    let text_y_size = TEXT_X_SIZE * y_ratio;
    let text_y_offset = TEXT_X_OFFSET * y_ratio;
    let line_width_y_offset = LINE_WIDTH_X_OFFSET * y_ratio;

    let x_scale = MATRIX_X_SIZE / x_len;
    let y_scale = matrix_y_size / y_len;


    let mut matrix: Vec<Vec<f64>> = vec![
        vec![0.0; MATRIX_LEN]; (matrix_y_size as usize) + 1
    ];

    let scaled_points: Vec<((f64, f64), bool)> = points
        .iter()
        .map(|point| {
            let x_scaled = ((point.x - min_x) * x_scale) + TEXT_X_OFFSET;
            let y_scaled = ((point.y - min_y) * y_scale) + text_y_offset;

            ((x_scaled, y_scaled), point.new_line)
    }).collect();

    let mut last_x = 0.0;
    let mut last_y = 0.0;

    for ((current_x, current_y), newline) in scaled_points {

        if !newline {
            let curr_x_start = current_x - LINE_WIDTH_X_OFFSET;
            let curr_x_end = current_x + LINE_WIDTH_X_OFFSET;

            let last_x_start = last_x - LINE_WIDTH_X_OFFSET;
            let last_x_end = last_x + LINE_WIDTH_X_OFFSET;

            let top_y = current_y.max(last_y) + line_width_y_offset;
            let bottom_y = current_y.min(last_y) - line_width_y_offset; 

            let start_x = (last_x_start.min(curr_x_start)) as usize;
            let end_x = (last_x_end.max(curr_x_end)) as usize + 1;


            for x in start_x..(end_x + 1) {

                let left_line_y = line(x as f64, (last_x_start, last_y), (curr_x_start, current_y));
                let right_line_y = line(x as f64, (last_x_end, last_y), (curr_x_end, current_x));

                let top_line_y = left_line_y
                    .max(right_line_y)
                    .min(top_y) as usize;
                let bottom_line_y = left_line_y
                    .min(right_line_y)
                    .max(bottom_y) as usize;

                for y in bottom_line_y..(top_line_y + 1) {
                    matrix[y][x] = 1.0;
                }

            }
        }

        last_x = current_x;
        last_y = current_y;
    }

    vec![matrix]

}
