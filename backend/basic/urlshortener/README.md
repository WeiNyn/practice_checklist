# URL Shortener

A simple URL shortener service built with Rust, Axum, and SQLx (SQLite).

## Features

*   Shorten long URLs into a compact base-62 encoded string.
*   Redirect short URLs to their original destination.
*   Track the number of clicks for each short URL.
*   List all shortened URLs with pagination.
*   Clean up (delete) URLs that haven't been used for a specified number of days.
*   Basic URL validation (must start with `http://` or `https://`).
*   Logging and tracing for requests.

## Technologies Used

*   **Rust** (Edition 2024)
*   **Axum**: Web framework
*   **SQLx**: SQL toolkit for Rust, using SQLite as the database
*   **Tokio**: Asynchronous runtime
*   **Serde**: Serialization/deserialization
*   **base-62**: For encoding IDs into short URL strings
*   **tower-http**: For HTTP middleware (tracing)
*   **tracing-subscriber**: For application-level tracing

## API Endpoints

The server runs on `http://localhost:3000` by default.

*   **`GET /`**
    *   Description: Welcome message.
    *   Response: `Welcome to the URL Shortener!`

*   **`POST /create`**
    *   Description: Creates a new short URL.
    *   Request Body (JSON):
        ```json
        {
            "original_url": "your_long_url_here"
        }
        ```
    *   Success Response (200 OK, text/plain): The short URL string (e.g., `AaBbcC`).
    *   Error Responses:
        *   `400 Bad Request`: If the URL is empty or invalid (e.g., does not start with `http://` or `https://`).
        *   `500 Internal Server Error`: If there's an issue creating or storing the URL.

*   **`GET /{short_url}`**
    *   Description: Redirects to the original URL corresponding to the `short_url` and increments its click count.
    *   Parameters:
        *   `short_url` (path): The base-62 encoded short URL string.
    *   Success Response: `307 Temporary Redirect` to the original URL.
    *   Error Responses:
        *   `404 Not Found`: If the short URL doesn't exist.
        *   `500 Internal Server Error`: If there's a database issue.

*   **`GET /urls`**
    *   Description: Retrieves a list of all stored URLs.
    *   Query Parameters:
        *   `limit` (integer, optional): Maximum number of URLs to return.
        *   `offset` (integer, optional): Number of URLs to skip (for pagination).
    *   Success Response (200 OK, JSON): An array of URL objects. Each object includes `id`, `original_url`, `short_url`, `click_count`, and `created_at`.
        ```json
        [
            {
                "id": 1,
                "original_url": "https://example.com/very/long/url",
                "short_url": "AaBb",
                "click_count": 10,
                "created_at": "2025-05-26T10:00:00Z"
            },
            // ... more URLs
        ]
        ```
    *   Error Response: `500 Internal Server Error`.

*   **`GET /clicks/{short_url}`**
    *   Description: Gets the current click count for a specific short URL.
    *   Parameters:
        *   `short_url` (path): The base-62 encoded short URL string.
    *   Success Response (200 OK, JSON): The click count as an integer.
        ```json
        10
        ```
    *   Error Responses:
        *   `404 Not Found`: If the short URL doesn't exist.
        *   `500 Internal Server Error`.

*   **`DELETE /cleanup`**
    *   Description: Deletes URLs that have not been clicked (i.e., `click_count` is 0 or `last_clicked_at` is older than the specified `days`) for a given number of days.
    *   Query Parameters:
        *   `days` (integer, required): The number of days of inactivity after which a URL is considered unused.
    *   Success Response (200 OK, JSON): The number of URLs deleted.
        ```json
        5
        ```
    *   Error Response: `500 Internal Server Error`.

## How to Run

1.  **Prerequisites**:
    *   Rust toolchain installed.
    *   SQLite installed (the database file `url.db` will be created automatically if it doesn't exist).

2.  **Clone the repository (if applicable) and navigate to the `backend/basic/urlshortener` directory.**

3.  **Build the project**:
    ```bash
    cargo build
    ```

4.  **Run the application**:
    ```bash
    cargo run
    ```
    The server will start on `http://localhost:3000`.

## Database

*   The application uses SQLite. The database file is named `url.db` and is created in the project's root directory (`backend/basic/urlshortener/`).
*   Database schema and migrations are handled by SQLx. The `init_db` function in `src/storage.rs` likely sets up the database and runs migrations on startup.
*   The `migrations` directory (`backend/basic/urlshortener/migrations/`) contains the SQL migration files.
