use std::env;
use std::io;
use std::fs::File;
use std::io::Write;
use std::fs;
use image::imageops::replace;
use image::io::Reader as ImageReader;
use std::process;
use image::{GenericImage, Rgb, GenericImageView, ImageBuffer, RgbImage, GrayImage, Luma, Rgba, DynamicImage, Pixel};
use imageproc::edges;
use imageproc::gray_image;
use inline_python::python;
use imageproc::filter::gaussian_blur_f32;
use std::thread;

struct PixelExt {
    color: (u8,u8,u8),
    marked: bool,
    x: u32,
    y: u32,
}

fn flood_fill(image: RgbImage) {
    //start with sorted array of all pixels (reading order)

    let (x,y) = image.dimensions(); 
    let mut pixel_vec = Vec::new();
    for curr_x in 0..x {
        for curr_y in 0..y {
            //initialize struct and push to vec for easy data access later
            let color = image.get_pixel(x,y).to_rgb();
            let (r, g, b) = (color[0], color[1], color[2]);
            let pixel_ext = PixelExt { color: (r,g,b), marked: false, x: x, y: y };
            pixel_vec.push(pixel_ext);
        }
    } 
    
   //now loop through the pixel vec checking for same color pixels and add them to a vec 

    //bounds check
    

     
    //now flood fill from 0,0 -- (first pixel in pixel_vec)
    
        //check surrounding pixels of first pixel
        //if same color check surrounding pixels & mark
    
    //what constitutes a floodfill? -- check all surrounding pixels for same color until


    //

}


fn normalize((r, g, b): (u8, u8, u8), divisor: f32) -> (u8, u8, u8) {

    let (mut ret_r, mut ret_g, mut ret_b): (u8, u8, u8);
    
    let mut div = (r as f32 / divisor).round();
    if (div < 0.0) || (div > (255.0 / div)) {
        panic!("Error with normalizer");
    }
    if div == 0.0 {
        ret_r = 1;
    }
    else {
        ret_r = (div * divisor) as u8;
    }

    div = (g as f32 / divisor).round();
    if (div < 0.0) || (div > (255.0 / div)) {
        panic!("Error with normalizer");
    }
    if div == 0.0 {
        ret_g = 1;
    }
    else {
        ret_g = (div * divisor) as u8;
    }

    div = (b as f32 / divisor).round();
    if (div < 0.0) || (div > (255.0 / div)) {
        panic!("Error with normalizer");
    }
    if div == 0.0 {
        ret_b = 1;
    }
    else {
        ret_b = (div * divisor) as u8;
    } 


    (ret_r, ret_g, ret_b)
}

fn set_colors(original_img: RgbImage, color_divisor: f32) -> RgbImage {
    let (x,y) = original_img.dimensions();
    let mut ret = RgbImage::new(x,y);

    for rel_x in 0..x {
        for rel_y in 0..y {
            //for all pixels
            let pixel = original_img.get_pixel(rel_x, rel_y); 
            let (r, g, b) = {
                (pixel[0], pixel[1], pixel[2])
            };

            let (new_r, new_g, new_b) = normalize((r,g,b), color_divisor);
            
            ret.put_pixel(rel_x, rel_y, Rgb([new_r, new_g, new_b]));
        }
    }
   ret 
}

fn check_pixel(image: &RgbImage, x: u32, y: u32) -> (u8, u8, u8) {
    //checks surrounding pixels and returns whether its an edge or not

    let curr_pixel = image.get_pixel(x,y).to_rgb();
    let curr_pixel_int = curr_pixel[0] as i32 + curr_pixel[1] as i32 + curr_pixel[2] as i32;

    //initialize surrounding pixels
    let mut around: Vec<(u8,u8,u8)> = Vec::new();
    let top = image.get_pixel(x, y-1).to_rgb(); //top pixel
    let bottom = image.get_pixel(x, y+1).to_rgb(); //bottom pixel
    let bottom_left = image.get_pixel(x-1, y+1).to_rgb(); //bottom left pixel
    let left = image.get_pixel(x-1, y).to_rgb(); //left pixel
    let top_left = image.get_pixel(x-1, y-1).to_rgb(); //top left pixel
    let top_right = image.get_pixel(x+1, y-1).to_rgb(); //top right pixel
    let right = image.get_pixel(x+1, y).to_rgb(); //right pixel
    let bottom_right = image.get_pixel(x+1, y+1).to_rgb(); //bottom right pixel
    
    around.push((top[0], top[1], top[2]));
    around.push((bottom[0], bottom[1], bottom[2] ));
    around.push((bottom_left[0], bottom_left[1], bottom_left[2]));
    around.push((left[0], left[1], left[2]));
    around.push((top_left[0], top_left[1], top_left[2]));
    around.push((top_right[0], top_right[1], top_right[2]));
    around.push((right[0], right[1], right[2]));
    around.push((bottom_right[0], bottom_right[1], bottom_right[2]));
    
    //get value as int to calculate difference
    let mut as_int: Vec<i32> = Vec::new();
    for i in around.iter() {
        //println!(" vec -- {} {} {}", i.0, i.1, i.2);
        as_int.push((i.0 as i32 + i.1 as i32 + i.2 as i32));
    }

    //check if different and add to diff_count
    let mut diff_count = 0;
    let mut same_count = 0;

    for i in 0..8 {
        if((curr_pixel_int != as_int[i]) && (as_int[i] != 0)) {
            diff_count += 1;
        }
        else {
            same_count += 1;
        }
    }

    if diff_count < 2 {
        //println!("{}", same_count);
        (curr_pixel[0], curr_pixel[1], curr_pixel[2])
    }
    else {
        //println!("curr: {}, diff_count: {}", curr_pixel_int, diff_count);
        (0, 0, 0)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect(); 

    if args.len() > 3 {
        println!("USAGE:\n./exec 'prompt' 'save_as'\n");
        process::exit(-1);
    }

    let mut dyn_img: DynamicImage;
    let arg1 = args[1].to_string();
    if arg1 == "-f".to_string() {
        dyn_img = ImageReader::open(args[2].to_string()).unwrap().decode().unwrap();
    }
    else {
       //call to DALLE using environment variable to get img saved in same dir
        let api_prompt = args[1].to_string();
        let count_as_str = fs::read_to_string("./pics/count.txt").unwrap();
        let mut as_int = count_as_str.trim().parse::<i32>().unwrap();
        as_int += 1;
        let save_as = String::from(as_int.to_string());
        let path = format!("./pics/{}.png", as_int);
        let path_clone = path.clone();

        python! {
            import openai
            import requests
            
            openai.api_key = "atotallyworkingkey"

            new_image = openai.Image.create(
                    prompt='api_prompt,
                    n=1,
                    size="1024x1024"
                )
            image_url = new_image["data"][0]["url"]
            res = requests.get(image_url).content
            with open('path_clone, "wb") as handler:
                handler.write(res)
        }


        //update count in file
        let mut file = File::create("./pics/count.txt").unwrap();
        file.write_all(save_as.as_bytes()).expect("Failed to write new value to count");

        dyn_img = ImageReader::open(path).unwrap().decode().unwrap(); 
    }

    //option code complete
    let img = dyn_img.to_rgb8();
    let (x,y) = img.dimensions();
    let mut tuple_vec: Vec<(u8, u8, u8)> = Vec::new();
    let mut individual_color_count = 0;
    
    let mut imgbuf = GrayImage::new(x,y);
    let mut actual_img = RgbImage::new(x,y);


    //create slice buffer
    let mut slice_buff:Vec<RgbImage> = Vec::new();

    //create new coords for multithread
    let x1 = x/4;
    let y1 = y/4;


    let blurred_img = gaussian_blur_f32(&img, 1.5);

    actual_img = set_colors(blurred_img, 17.0); 

    //let blurred_img = gaussian_blur_f32(&blurred_img, 2.0);
    
    //save blurred and normalized img
    
    //actual_img = set_colors(blurred_img, 17.0);  
    //let blurred_img = gaussian_blur_f32(&actual_img, 2.0);
    //actual_img = set_colors(blurred_img, 17.0);
    actual_img.save("filtered.png").unwrap();

    //check surrounding pixels and mark them
    
    for mut curr_x in 1..(x-1) {
        for mut curr_y in 1..(y-1) {

            let u = check_pixel(&actual_img, curr_x, curr_y);
            let replacement = Rgb::from([u.0,u.1,u.2]);
            actual_img.put_pixel(curr_x, curr_y, replacement);
        }
    }
    let mut black_white = RgbImage::new(x,y);
    for curr_x in 0..x {
        for curr_y in 0..y {
            let u = actual_img.get_pixel(curr_x, curr_y);
            let p = u[0] as i32 + u[1] as i32 + u[2] as i32;
            if p == 0 {
                black_white.put_pixel(curr_x, curr_y, Rgb::from([255, 255, 255]));
            }
            else {
                black_white.put_pixel(curr_x, curr_y, Rgb::from([0,0,0]));
            }
        }
    }
    black_white.save("black_white.png").unwrap();


    
    
    /*let u = check_pixel(&actual_img, 20, 5);
    let replacement = Rgb::from([u.0,u.1,u.2]);
    actual_img.put_pixel(20, 5, replacement);
*/
    /*
    
    let mut palette = RgbImage::new(individual_color_count, 100);
    println!("{} Individual Colors", individual_color_count); 

    // Add a new variable to keep track of the y coordinate
    let mut x = 0;
    let mut y = 0;
    
    

    for rgb in tuple_vec.iter() {
        let r = rgb.0;
        let g = rgb.1;
        let b = rgb.2;
        for i in 0..9 {
            // Check if the x coordinate is at the end of the image
        // and move to the next row if necessary
        if x + i >= individual_color_count {
            x = 0;
            y += 5;
        }
        for p in 0..5 {
            palette.put_pixel(x + i, y + p, Rgb([r,g,b]));
        }
    }
    x += 9;
        
}
    */
    



    //search for pixel edges
    //loop over every pixel and searching around them for edges seems slow
    //
    //
    //let mut edges = imageproc::edges::canny(&imgbuf, 16.0, 33.0);
    //let mut edges_med = imageproc::edges::canny(&imgbuf, 10.0, 15.0);
    //let mut edges_low_contrast = imageproc::edges::canny(&imgbuf, 1.0, 3.0);

    //change swap blacks and whites for printing
    /*
    let (x1, y1) = edges.dimensions();
    for rel_x in 0..x1{ 
        for rel_y in 0..y1 { 
            if edges_low_contrast.get_pixel(rel_x, rel_y)[0] == 255 {
                edges_low_contrast.put_pixel(rel_x, rel_y, Luma::from([0]));
            }
            else {
                edges_low_contrast.put_pixel(rel_x, rel_y, Luma::from([255]));
            }

            if edges.get_pixel(rel_x, rel_y)[0] == 255 {
                edges.put_pixel(rel_x, rel_y, Luma::from([0]));  
            }
            else {
                edges.put_pixel(rel_x, rel_y, Luma::from([255]));
            }

            if edges_med.get_pixel(rel_x, rel_y)[0] == 255 {
                edges_med.put_pixel(rel_x, rel_y, Luma::from([0]));
            }
            else {
                edges_med.put_pixel(rel_x, rel_y, Luma::from([255]));
            }
        }
    }*/
    //edges.save("./lines/high.png").unwrap();
    //edges_med.save("./lines/med.png").unwrap();
    //edges_low_contrast.save("./lines/low.png").unwrap();

    //palette.save("palette.png").unwrap(); 
    //let actual_img = gaussian_blur_f32(&actual_img, 3.0);
    actual_img.save("line_test.png").unwrap();
}

