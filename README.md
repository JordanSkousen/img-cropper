# img-cropper

This Node.js application allows you to crop images from a specified directory to a given size.

**Usage:**

1.  **Create an input directory:**
    Create a folder named `input` (or any name you prefer) in the project's root directory (`/Users/jordan/source/repos/img-cropper/`) and place the images you want to crop inside it.

2.  **Run the application:**
    Open your terminal in the project's root directory and execute the following command, replacing `input` with your input directory name, `output` with your desired output directory name, and `400x300` with your desired crop size (width x height):

    ```bash
    npm start -- -i input -o output -s 400x300
    ```

    *   `-i` or `--inputDir`: Path to the directory containing your images.
    *   `-o` or `--outputDir`: Path to the directory where cropped images will be saved. This directory will be created if it doesn't exist.
    *   `-s` or `--size`: The desired crop size in `WxH` format (e.g., `400x300`).

**Example:**

If you have images in `/Users/jordan/source/repos/img-cropper/my_images` and want to crop them to `800x600` and save them to `/Users/jordan/source/repos/img-cropper/cropped_images`, you would run:

```bash
npm start -- -i my_images -o cropped_images -s 800x600
```

The application will create `cropped_images` directory if it doesn't exist and save the cropped versions of your images there, prefixed with `cropped_`.
