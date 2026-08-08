use topcoat::{
    Result,
    asset::{Asset, asset},
    context::Cx,
    router::{
        content::Json, 
        Router, RouterBuilderDiscoverExt, 
        page, route
    },
    view::{component, view},
};
mod mod_todo;

#[derive(serde::Deserialize)] 
struct CreateUser { 
    name: String 
}
#[derive(serde::Serialize)] 
struct User { 
    name: String 
}
#[derive(serde::Deserialize)] 
struct CreateTodo { 
    title: String 
}

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build()).await.unwrap();
}

#[route(POST "/api/todo/create")]
async fn create_todo(cx: &Cx, Json(input): Json<CreateTodo>
) -> Result<String> {
    println!("title={}", input.title);

    let title = input.title;
    mod_todo::add_todo(&title);
    Ok("OK".to_string())
}

#[route(GET "/api/todo/list")]
async fn list_todo() -> Result<String> {
    let resp = mod_todo::list_todo_json();
    match resp {
        Ok(value) => {
            println!("結果: {}", value);
            return Ok(value);
        },
        Err(err) => {
            println!("エラー: {}", err);
            return Ok(err);
        },
    }    
    Ok("OK".to_string())
}

#[route(POST "/api/users")]
async fn create_user(cx: &Cx, Json(input): Json<CreateUser>
) -> Result<Json<User>> {
    println!("name={}", input.name);
    Ok(Json(User { name: input.name }))
}

#[route(GET "/api/health")]
async fn health() -> Result<&'static str> {
    Ok("ok, /api/health")
}

#[page("/")]
async fn home() -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Hello world"</title>
                topcoat::dev::script()
            </head>
            <body>
                hello(name: "World-!!!")
            </body>
        </html>
    }
}

#[component]
async fn hello(name: &str) -> Result {
    view! {
        <h1>"Hello, " (name) "!"</h1>
    }
}