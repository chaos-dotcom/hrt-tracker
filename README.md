# HRT Tracker

A self-hosted web application (that could have been a spreadsheet) to track hormone replacement therapy (HRT) data, including dosages, blood test results, and body measurements. Built with SvelteKit.

## Features

-   **Dosage Tracking:** Record historical doses and set up medication schedules for injectable estradiol, oral estradiol, antiandrogens, and progesterone.
-   **Blood Test Monitoring:** Log detailed blood test results, including levels for Estradiol, Testosterone, Progesterone, FSH, LH, Prolactin, and SHBG.
-   **Measurement History:** Keep track of physical changes with measurements for weight, height, bust, waist, hips, and more.
-   **Data Visualization:** View your hormone levels and dosage history on an interactive chart.
-   **Estrannaise Integration:** Generate a link to model your injectable estradiol regimen on [Estrannaise](https://estrannai.se/).
-   **Data Portability:** Easily back up your data to a JSON file and restore it when needed.
-   **Private:** Your data is saved to a `hrt-data.json` file on the server, ensuring you have full control over your information.

## Getting Started

To run this project locally, you'll need Node.js and npm (or bun) installed.

1.  Clone this repository.

2.  Install dependencies:
    ```bash
    npm install
    ```
    or if you use bun:
    ```bash
    bun install
    ```

3.  Run the development server:
    ```bash
    npm run dev
    ```
    or
    ```bash
    bun dev
    ```

4.  Open your browser and navigate to `http://localhost:5173`.

## Tech Stack
- RUST
- ESOTERIC RUST SHIT for WASM reasons