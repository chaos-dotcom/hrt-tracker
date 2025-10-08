# HRT Tracker

A simple, self-hosted application to track Hormone Replacement Therapy (HRT) progress.

## Planned Features

- **Data Storage**:
  - Uses local storage for simplicity. No login or accounts required.
  - Option to import/export data as a JSON file for backup or transfer.
  - (Optional) Support for remote storage via a self-hosted server.
- **Onboarding**:
  - Simple setup to input current dosage and past blood results.
- **Tracking**:
  - Track dosages (injections, oral, etc.).
  - Log blood test results.
  - Track various configurable measurements over time.
- **Visualization**:
  - View changes in graphs over time for trackable data.
- **Future Ideas**:
  - Set reminders for taking HRT.
  - Read blood test results from a PDF or image using OCR.
  - Theme options (e.g., Rosé Pine).

## Routes

-   `/`: Dashboard view after setup, with buttons to add blood tests or log dosages. Onboarding for new users.
-   `/create/blood-test`: Create a new blood test entry.
-   `/create/dosage`: Log a new HRT dosage.
-   `/tracker`: View historical blood tests and dosages.
-   `/settings`: Configuration options, like theme selection.
-   `/backup`: Import or export data to a JSON file.

## Getting Started

This project is designed to be run with Docker.

1.  Clone the repository.
2.  Run the application using Docker Compose:
    ```sh
    docker-compose up -d
    ```
3.  The application will be available at `http://localhost:3000`.
