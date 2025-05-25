# Todo App API

This is a RESTful API for managing a TODO list, built with Rust and the Axum framework, using SQLite as the database.

## Features

*   Create, Read, Update, and Delete TODO items.
*   Get all TODO items.
*   Get a specific TODO item by its ID.
*   Filter TODO items by completion status (completed or incomplete).
*   Filter TODO items within a specified time range.
*   Mark TODO items as complete or incomplete.
*   Update the title and description of TODO items.

## Project Structure

*   `Cargo.toml`: Defines project dependencies and metadata.
*   `src/main.rs`: Contains the main application logic, including route definitions and request handlers.
*   `src/storage.rs`: Handles database interactions, including initializing the database and functions for CRUD operations.
*   `migrations/`: Contains SQL migration scripts for setting up and updating the database schema.

## Database Schema

The `todo` table has the following columns:

*   `id`: INTEGER (Primary Key, Autoincrement)
*   `title`: TEXT (Not Null)
*   `description`: TEXT
*   `completed`: BOOLEAN (Not Null, Default: 0)
*   `created_at`: TIMESTAMP (Default: CURRENT_TIMESTAMP)
*   `updated_at`: TIMESTAMP (Default: CURRENT_TIMESTAMP)

Indexes are created on `completed` and `created_at` columns.
Triggers are in place to:
*   Automatically update the `updated_at` timestamp when a todo item is modified.
*   (Note: The trigger `delete_completed_todos` seems to attempt to delete a todo again after it's already been deleted if it was completed. This might be unintentional or have a specific purpose not immediately obvious from the schema.)

## API Endpoints

All endpoints are prefixed with `/todos`.

*   `GET /`: Returns "Hello, World!"
*   `GET /health`: Returns "OK" - can be used for health checks.
*   `GET /todos`: Retrieves all TODO items.
*   `POST /todos`: Creates a new TODO item.
    *   Request Body (JSON):
        ```json
        {
            "title": "String",
            "description": "Optional<String>"
        }
        ```
*   `GET /todos/{id}`: Retrieves a specific TODO item by its ID.
*   `PUT /todos/{id}`: Updates a specific TODO item by its ID.
    *   Request Body (JSON):
        ```json
        {
            "title": "Optional<String>",
            "description": "Optional<String>",
            "completed": "Optional<bool>"
        }
        ```
*   `DELETE /todos/{id}`: Deletes a specific TODO item by its ID.
*   `GET /todos/complete`: Retrieves all completed TODO items.
*   `GET /todos/incomplete`: Retrieves all incomplete TODO items.
*   `POST /todos/time-range`: Retrieves TODO items created within a specific time range.
    *   Request Body (JSON):
        ```json
        {
            "start": "String (ISO 8601 format)",
            "end": "String (ISO 8601 format)"
        }
        ```

## Setup and Installation

1.  **Clone the repository (if applicable).**
2.  **Ensure you have Rust installed.** If not, follow the instructions at [rust-lang.org](https://www.rust-lang.org/).
3.  **Ensure you have SQLite installed.**
4.  **Set up the database:**
    *   The application uses `sqlx-cli` for migrations. If you don't have it, install it:
        ```bash
        cargo install sqlx-cli --no-default-features --features native-tls,sqlite
        ```
    *   Create a `.env` file in the `backend/basic/todoapp/` directory with the database URL:
        ```env
        DATABASE_URL=sqlite://todoapp.db
        ```
    *   Run the migrations:
        ```bash
        cd backend/basic/todoapp
        sqlx database create # Might not be needed if the db file is created automatically
        sqlx migrate run
        ```
5.  **Build the project:**
    ```bash
    cd backend/basic/todoapp
    cargo build
    ```

## How to Run

1.  Navigate to the `backend/basic/todoapp` directory.
2.  Run the application:
    ```bash
    cargo run
    ```
    The server will start on `http://0.0.0.0:3000`.

## How to Test

The project includes unit tests in `src/storage.rs`.

1.  Navigate to the `backend/basic/todoapp` directory.
2.  Run the tests:
    ```bash
    cargo test
    ```
    This will use a separate `test.db` for testing purposes, which is cleaned up afterwards.
