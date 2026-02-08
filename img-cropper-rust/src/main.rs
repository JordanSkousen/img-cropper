use clap::Parser;
use image::ImageFormat;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing images
    #[clap(short = 'i', long)]
    input_dir: PathBuf,

    /// Output directory for cropped images
    #[clap(short = 'o', long)]
    output_dir: PathBuf,

    /// Crop size in WxH format (e.g., 400x300)
    #[clap(short = 's', long)]
    size: String,

    /// Number of parallel instances
    #[clap(short = 'c', long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=64))]
    instances: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let start_time = Instant::now();

    let (width, height) = parse_size(&args.size)?;

    if !args.input_dir.exists() {
        eprintln!("Error: Input directory not found: {:?}", args.input_dir);
        std::process::exit(1);
    }

    if !args.output_dir.exists() {
        fs::create_dir_all(&args.output_dir)?;
        println!("Created output directory: {:?}", args.output_dir);
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.instances as usize)
        .build_global()?;

    println!("Processing images from: {:?}", args.input_dir);
    println!("Cropping to size: {}x{}", width, height);
    println!("Using {} parallel instances.", args.instances);
    println!("Saving to: {:?}", args.output_dir);

    let processed_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);

    let image_paths: Vec<_> = WalkDir::new(&args.input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let input_file_path = entry.path();
            let extension = input_file_path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_ascii_lowercase());

            if let Some(ext) = extension {
                if is_supported_image_extension(&ext) {
                    let output_file_name = input_file_path.file_name()?;
                    let output_file_path = args.output_dir.join(output_file_name);
                    return Some((input_file_path.to_path_buf(), output_file_path));
                }
            }
            None
        })
        .collect();

    image_paths.par_iter().for_each(|(input_file_path, output_file_path)| {
        match crop_image(input_file_path, output_file_path, width, height) {
            Ok(_) => {
                println!(
                    "Cropped: {:?} -> {:?}",
                    input_file_path, output_file_path
                );
                processed_count.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                eprintln!(
                    "Error cropping {:?}: {}",
                    input_file_path,
                    e.to_string()
                );
                failed_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let elapsed_time = start_time.elapsed();
    println!("Image cropping complete in {:.2?}.", elapsed_time);
    println!(
        "Processed {} images, failed to process {} images.",
        processed_count.load(Ordering::Relaxed),
        failed_count.load(Ordering::Relaxed)
    );

    Ok(())
}

fn parse_size(size_str: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err("Invalid size format. Please use WxH (e.g., 400x300).".to_string());
    }

    let width = parts[0]
        .parse::<u32>()
        .map_err(|_| "Invalid width. Must be a positive integer.".to_string())?;
    let height = parts[1]
        .parse::<u32>()
        .map_err(|_| "Invalid height. Must be a positive integer.".to_string())?;

    if width == 0 || height == 0 {
        return Err("Width and height must be positive integers.".to_string());
    }

    Ok((width, height))
}

fn is_supported_image_extension(ext: &str) -> bool {
    matches!(ext, "jpg" | "jpeg" | "png" | "gif" | "webp")
}

fn crop_image(
    input_path: &Path,
    output_path: &Path,
    target_width: u32,
    target_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageReader, GenericImageView};
    let img = ImageReader::open(input_path)?
        .decode()?;

    let (original_width, original_height) = img.dimensions();

    let img_resized = if original_width * target_height > original_height * target_width {
        // Original image is wider than the target aspect ratio,
        // so we resize based on height and then crop width
        img.resize(
            (original_width * target_height) / original_height,
            target_height,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        // Original image is taller than or has the same aspect ratio as the target,
        // so we resize based on width and then crop height
        img.resize(
            target_width,
            (original_height * target_width) / original_width,
            image::imageops::FilterType::Lanczos3,
        )
    };

    let (resized_width, resized_height) = img_resized.dimensions();

    // Calculate the coordinates for center crop
    let crop_x = (resized_width.saturating_sub(target_width)) / 2;
    let crop_y = (resized_height.saturating_sub(target_height)) / 2;

    let cropped_img = img_resized.crop_imm(crop_x, crop_y, target_width, target_height);

    // Determine the image format based on the output file extension
    let format = ImageFormat::from_path(output_path)
        .unwrap_or(ImageFormat::Png); // Default to PNG if unknown

    cropped_img.save_with_format(output_path, format)?;

    Ok(())
}
