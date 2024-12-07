use gloo_net::http::Request;
use super::Employee;

pub async fn fetch_employees() -> Result<Vec<Employee>, String> {
    Request::get("/api/employees")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_employee(employee: &Employee) -> Result<(), String> {
    Request::post("/api/employees")
        .json(employee)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}