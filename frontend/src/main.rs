use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures;
mod models;
use models::Employee;

#[function_component(App)]
fn app() -> Html {
    let employees = use_state(Vec::new);
    let error = use_state(|| None);

    {
        let employees = employees.clone();
        let error = error.clone();  // Klona error här
        use_effect_with_deps(move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let response = Request::get("/api/employees")
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<Vec<Employee>>().await {
                            employees.set(data);
                        }
                    },
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
            || ()
        }, ());
    }

    html! {
        <div class="container">
            <h1>{"Employee Management"}</h1>
            <div class="employee-list">
                <h2>{"Employees"}</h2>
                {if let Some(err) = (*error).as_ref() {
                    html! { <div class="error">{ err }</div> }
                } else {
                    html! {
                        <table>
                            <thead>
                                <tr>
                                    <th>{"ID"}</th>
                                    <th>{"Name"}</th>
                                    <th>{"Salary"}</th>
                                    <th>{"Region"}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {(*employees).iter().map(|emp| {
                                    let Employee { employee_id, name, salary, region } = emp;
                                    html! {
                                        <tr key={*employee_id}>
                                            <td>{employee_id}</td>
                                            <td>{name}</td>
                                            <td>{salary}</td>
                                            <td>{region}</td>
                                        </tr>
                                    }
                                }).collect::<Html>()}
                            </tbody>
                        </table>
                    }
                }}
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}