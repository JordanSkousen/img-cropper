const yargs = require('yargs');
const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

console.log('process.argv:', process.argv);

const argv = yargs(process.argv.slice(2))
  .option('inputDir', {
    alias: 'i',
    description: 'Input directory containing images',
    type: 'string',
    demandOption: true,
  })
  .option('outputDir', {
    alias: 'o',
    description: 'Output directory for cropped images',
    type: 'string',
    demandOption: true,
  })
  .option('size', {
    alias: 's',
    description: 'Crop size in WxH format (e.g., 400x300)',
    type: 'string',
    demandOption: true,
  })
  .help()
  .alias('help', 'h')
  .argv;

async function cropImages() {
  const { inputDir, outputDir, size } = argv;
  const [width, height] = size.split('x').map(Number);

  if (isNaN(width) || isNaN(height) || width <= 0 || height <= 0) {
    console.error('Invalid size format. Please use WxH (e.g., 400x300) with positive integers.');
    process.exit(1);
  }

  if (!fs.existsSync(inputDir)) {
    console.error(`Input directory not found: ${inputDir}`);
    process.exit(1);
  }

  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
    console.log(`Created output directory: ${outputDir}`);
  }

  console.log(`Processing images from: ${inputDir}`);
  console.log(`Cropping to size: ${width}x${height}`);
  console.log(`Saving to: ${outputDir}`);

  try {
    const files = await fs.promises.readdir(inputDir);
    const imageFiles = files.filter(file => {
      const ext = path.extname(file).toLowerCase();
      return ['.jpg', '.jpeg', '.png', '.gif', '.webp'].includes(ext);
    });

    if (imageFiles.length === 0) {
      console.log('No supported image files found in the input directory.');
      return;
    }

    for (const file of imageFiles) {
      const inputFilePath = path.join(inputDir, file);
      const outputFilePath = path.join(outputDir, file);

      try {
        await sharp(inputFilePath)
          .resize(width, height, {
            fit: sharp.fit.cover, // Crop to cover the area, maintaining aspect ratio
            position: sharp.gravity.center, // Crop from the center
          })
          .toFile(outputFilePath);
        console.log(`Cropped: ${inputFilePath} -> ${outputFilePath}`);
      } catch (error) {
        console.error(`Failed to crop ${file}: ${error.message}`);
      }
    }
    console.log('Image cropping complete!');
  } catch (error) {
    console.error(`Error reading input directory: ${error.message}`);
    process.exit(1);
  }
}

cropImages();
