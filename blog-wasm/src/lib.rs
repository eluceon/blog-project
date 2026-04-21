use serde::{Deserialize, Serialize};

// ─── Shared data types (compile on all targets) ───────────────────────────────

/// A blog post as returned by the HTTP API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostData {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UserData {
    id: i64,
    username: String,
    email: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
    user: UserData,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ListPostsResponse {
    posts: Vec<PostData>,
    total: i64,
}

// ─── Yew application (wasm32 only) ───────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use yew::prelude::*;

/// Base URL of the blog HTTP API server.
#[cfg(target_arch = "wasm32")]
const SERVER_URL: &str = "http://localhost:8080";

/// Key used to store the JWT token in localStorage.
#[cfg(target_arch = "wasm32")]
const TOKEN_KEY: &str = "blog_token";

// ─── API helpers ──────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
async fn api_fetch_posts() -> Result<Vec<PostData>, String> {
    let resp = Request::get(&format!("{SERVER_URL}/api/posts?limit=50&offset=0"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("Server error {}", resp.status()));
    }
    let data: ListPostsResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.posts)
}

#[cfg(target_arch = "wasm32")]
async fn api_register(username: &str, email: &str, password: &str) -> Result<AuthResponse, String> {
    let body = serde_json::to_string(
        &serde_json::json!({"username": username, "email": email, "password": password}),
    )
    .map_err(|e| e.to_string())?;

    let resp = Request::post(&format!("{SERVER_URL}/api/auth/register"))
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Registration failed: {text}"));
    }
    resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn api_login(username: &str, password: &str) -> Result<AuthResponse, String> {
    let body = serde_json::to_string(
        &serde_json::json!({"username": username, "password": password}),
    )
    .map_err(|e| e.to_string())?;

    let resp = Request::post(&format!("{SERVER_URL}/api/auth/login"))
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Login failed: {text}"));
    }
    resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn api_create_post(token: &str, title: &str, content: &str) -> Result<PostData, String> {
    let body = serde_json::to_string(&serde_json::json!({"title": title, "content": content}))
        .map_err(|e| e.to_string())?;

    let resp = Request::post(&format!("{SERVER_URL}/api/posts"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Create post failed: {text}"));
    }
    resp.json::<PostData>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn api_update_post(
    token: &str,
    id: i64,
    title: &str,
    content: &str,
) -> Result<PostData, String> {
    let body = serde_json::to_string(&serde_json::json!({"title": title, "content": content}))
        .map_err(|e| e.to_string())?;

    let resp = Request::put(&format!("{SERVER_URL}/api/posts/{id}"))
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {token}"))
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Update post failed: {text}"));
    }
    resp.json::<PostData>().await.map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn api_delete_post(token: &str, id: i64) -> Result<(), String> {
    let resp = Request::delete(&format!("{SERVER_URL}/api/posts/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.ok() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Delete post failed: {text}"));
    }
    Ok(())
}

// ─── View state ───────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, PartialEq)]
enum View {
    Home,
    Login,
    Register,
}

// ─── Helpers to read DOM input/textarea values ────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn read_input(r: &NodeRef) -> String {
    r.cast::<web_sys::HtmlInputElement>()
        .map_or_else(String::new, |el| el.value())
}

#[cfg(target_arch = "wasm32")]
fn read_textarea(r: &NodeRef) -> String {
    r.cast::<web_sys::HtmlTextAreaElement>()
        .map_or_else(String::new, |el| el.value())
}

// ─── Root App component ───────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
#[function_component(App)]
fn app() -> Html {
    // ── Global state ──────────────────────────────────────────────────────────
    let token: UseStateHandle<Option<String>> = use_state(|| LocalStorage::get(TOKEN_KEY).ok());
    let user_id: UseStateHandle<Option<i64>> = use_state(|| None);
    let logged_username: UseStateHandle<Option<String>> = use_state(|| None);
    let posts: UseStateHandle<Vec<PostData>> = use_state(Vec::new);
    let view: UseStateHandle<View> = use_state(|| View::Home);
    let error: UseStateHandle<Option<String>> = use_state(|| None);
    // Incremented to trigger a post list reload without changing view
    let refresh: UseStateHandle<u32> = use_state(|| 0u32);
    // Which post is currently being edited (None = none)
    let editing_id: UseStateHandle<Option<i64>> = use_state(|| None);
    // Controlled inputs for the inline edit form
    let edit_title: UseStateHandle<String> = use_state(String::new);
    let edit_content: UseStateHandle<String> = use_state(String::new);

    // ── Form NodeRefs (uncontrolled — read on submit) ─────────────────────────
    let login_username_ref = use_node_ref();
    let login_password_ref = use_node_ref();
    let reg_username_ref = use_node_ref();
    let reg_email_ref = use_node_ref();
    let reg_password_ref = use_node_ref();
    let create_title_ref = use_node_ref();
    let create_content_ref = use_node_ref();

    // ── Load posts when on Home view or after a mutation ─────────────────────
    {
        let posts = posts.clone();
        let error = error.clone();
        let view_dep = (*view).clone();
        let refresh_dep = *refresh;
        use_effect_with((view_dep, refresh_dep), move |(v, _)| {
            if *v == View::Home {
                spawn_local(async move {
                    match api_fetch_posts().await {
                        Ok(p) => posts.set(p),
                        Err(e) => error.set(Some(e)),
                    }
                });
            }
        });
    }

    // ── Callbacks ─────────────────────────────────────────────────────────────

    let on_clear_error = {
        let error = error.clone();
        Callback::from(move |_: MouseEvent| error.set(None))
    };

    let on_logout = {
        let token = token.clone();
        let user_id = user_id.clone();
        let logged_username = logged_username.clone();
        let view = view.clone();
        Callback::from(move |_: MouseEvent| {
            token.set(None);
            user_id.set(None);
            logged_username.set(None);
            LocalStorage::delete(TOKEN_KEY);
            view.set(View::Home);
        })
    };

    let on_go_home = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set(View::Home))
    };

    let on_go_login = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set(View::Login))
    };

    let on_go_register = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set(View::Register))
    };

    // Login form submit
    let on_login_submit = {
        let token = token.clone();
        let user_id = user_id.clone();
        let logged_username = logged_username.clone();
        let view = view.clone();
        let error = error.clone();
        let login_username_ref = login_username_ref.clone();
        let login_password_ref = login_password_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let uname = read_input(&login_username_ref);
            let pass = read_input(&login_password_ref);
            if uname.is_empty() || pass.is_empty() {
                error.set(Some("Username and password are required.".to_owned()));
                return;
            }
            let (token, user_id, logged_username, view, error) = (
                token.clone(), user_id.clone(), logged_username.clone(),
                view.clone(), error.clone(),
            );
            spawn_local(async move {
                match api_login(&uname, &pass).await {
                    Ok(auth) => {
                        let _ = LocalStorage::set(TOKEN_KEY, &auth.token);
                        token.set(Some(auth.token));
                        user_id.set(Some(auth.user.id));
                        logged_username.set(Some(auth.user.username));
                        view.set(View::Home);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    // Register form submit
    let on_register_submit = {
        let token = token.clone();
        let user_id = user_id.clone();
        let logged_username = logged_username.clone();
        let view = view.clone();
        let error = error.clone();
        let reg_username_ref = reg_username_ref.clone();
        let reg_email_ref = reg_email_ref.clone();
        let reg_password_ref = reg_password_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let uname = read_input(&reg_username_ref);
            let email = read_input(&reg_email_ref);
            let pass = read_input(&reg_password_ref);
            if uname.is_empty() || email.is_empty() || pass.is_empty() {
                error.set(Some("All fields are required.".to_owned()));
                return;
            }
            let (token, user_id, logged_username, view, error) = (
                token.clone(), user_id.clone(), logged_username.clone(),
                view.clone(), error.clone(),
            );
            spawn_local(async move {
                match api_register(&uname, &email, &pass).await {
                    Ok(auth) => {
                        let _ = LocalStorage::set(TOKEN_KEY, &auth.token);
                        token.set(Some(auth.token));
                        user_id.set(Some(auth.user.id));
                        logged_username.set(Some(auth.user.username));
                        view.set(View::Home);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    // Create post form submit
    let on_create_submit = {
        let token = token.clone();
        let error = error.clone();
        let refresh = refresh.clone();
        let create_title_ref = create_title_ref.clone();
        let create_content_ref = create_content_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let title = read_input(&create_title_ref);
            let content = read_textarea(&create_content_ref);
            if title.is_empty() || content.is_empty() {
                error.set(Some("Title and content are required.".to_owned()));
                return;
            }
            let tok = match (*token).clone() {
                Some(t) => t,
                None => { error.set(Some("Not authenticated.".to_owned())); return; }
            };
            let (error, refresh) = (error.clone(), refresh.clone());
            spawn_local(async move {
                match api_create_post(&tok, &title, &content).await {
                    Ok(_) => refresh.set(*refresh + 1),
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    // ── Render ────────────────────────────────────────────────────────────────

    html! {
        <div class="app">
            // ── Navbar ────────────────────────────────────────────────────────
            <nav class="navbar">
                <span class="navbar-brand" onclick={on_go_home.clone()}>{"📝 Blog"}</span>
                <div class="nav-actions">
                    if token.is_some() {
                        <span class="user-info">
                            { format!("👤 {}", logged_username.as_deref().unwrap_or("User")) }
                        </span>
                        <button class="btn btn-outline" onclick={on_logout}>{"Logout"}</button>
                    } else {
                        <button class="btn" onclick={on_go_login.clone()}>{"Login"}</button>
                        <button class="btn btn-secondary" onclick={on_go_register.clone()}>
                            {"Register"}
                        </button>
                    }
                </div>
            </nav>

            // ── Error banner ──────────────────────────────────────────────────
            if let Some(msg) = (*error).clone() {
                <div class="error-banner">
                    { msg }
                    <button class="close-btn" onclick={on_clear_error}>{"✕"}</button>
                </div>
            }

            // ── Main content by view ──────────────────────────────────────────
            { match *view {

                // ── Home ──────────────────────────────────────────────────────
                View::Home => html! {
                    <div class="main-content">
                        // Create-post form (authenticated users only)
                        if token.is_some() {
                            <section class="create-post card">
                                <h2>{"New Post"}</h2>
                                <form onsubmit={on_create_submit}>
                                    <div class="form-group">
                                        <label>{"Title"}</label>
                                        <input ref={create_title_ref.clone()} type="text"
                                            class="form-input" placeholder="Post title" />
                                    </div>
                                    <div class="form-group">
                                        <label>{"Content"}</label>
                                        <textarea ref={create_content_ref.clone()}
                                            class="form-textarea" rows="4"
                                            placeholder="Write your post…" />
                                    </div>
                                    <button type="submit" class="btn">{"Publish"}</button>
                                </form>
                            </section>
                        }

                        // Post list
                        <section class="posts-section">
                            <h2>{"All Posts"}</h2>
                            if posts.is_empty() {
                                <p class="empty-state">
                                    {"No posts yet. Be the first to write one!"}
                                </p>
                            } else {
                                <div class="post-list">
                                { for posts.iter().map(|post| {
                                    let post_id = post.id;
                                    let is_editing = *editing_id == Some(post_id);
                                    let is_author = user_id
                                        .as_ref()
                                        .map(|uid| *uid == post.author_id)
                                        .unwrap_or(false);

                                    if is_editing {
                                        // ── Inline edit form ──────────────────
                                        let on_save = {
                                            let token = token.clone();
                                            let editing_id = editing_id.clone();
                                            let error = error.clone();
                                            let refresh = refresh.clone();
                                            let edit_title = edit_title.clone();
                                            let edit_content = edit_content.clone();
                                            Callback::from(move |e: SubmitEvent| {
                                                e.prevent_default();
                                                let title = (*edit_title).clone();
                                                let content = (*edit_content).clone();
                                                if title.is_empty() && content.is_empty() {
                                                    error.set(Some("Fields cannot both be empty.".to_owned()));
                                                    return;
                                                }
                                                let tok = match (*token).clone() {
                                                    Some(t) => t,
                                                    None => { error.set(Some("Not authenticated.".to_owned())); return; }
                                                };
                                                let (editing_id, error, refresh) =
                                                    (editing_id.clone(), error.clone(), refresh.clone());
                                                spawn_local(async move {
                                                    match api_update_post(&tok, post_id, &title, &content).await {
                                                        Ok(_) => {
                                                            editing_id.set(None);
                                                            refresh.set(*refresh + 1);
                                                        }
                                                        Err(e) => error.set(Some(e)),
                                                    }
                                                });
                                            })
                                        };
                                        let on_cancel = {
                                            let editing_id = editing_id.clone();
                                            Callback::from(move |_: MouseEvent| editing_id.set(None))
                                        };
                                        html! {
                                            <div class="post-card editing" key={post_id}>
                                                <form onsubmit={on_save}>
                                                    <div class="form-group">
                                                        <input type="text" class="form-input"
                                                            value={(*edit_title).clone()}
                                                            oninput={{
                                                                let edit_title = edit_title.clone();
                                                                Callback::from(move |ev: InputEvent| {
                                                                    // SAFETY: oninput fires on HtmlInputElement
                                                                    let el: web_sys::HtmlInputElement =
                                                                        ev.target_unchecked_into();
                                                                    edit_title.set(el.value());
                                                                })
                                                            }}
                                                        />
                                                    </div>
                                                    <div class="form-group">
                                                        <textarea class="form-textarea" rows="4"
                                                            value={(*edit_content).clone()}
                                                            oninput={{
                                                                let edit_content = edit_content.clone();
                                                                Callback::from(move |ev: InputEvent| {
                                                                    // SAFETY: oninput fires on HtmlTextAreaElement
                                                                    let el: web_sys::HtmlTextAreaElement =
                                                                        ev.target_unchecked_into();
                                                                    edit_content.set(el.value());
                                                                })
                                                            }}
                                                        />
                                                    </div>
                                                    <div class="post-actions">
                                                        <button type="submit" class="btn">{"Save"}</button>
                                                        <button type="button" class="btn btn-outline"
                                                            onclick={on_cancel}>{"Cancel"}</button>
                                                    </div>
                                                </form>
                                            </div>
                                        }
                                    } else {
                                        // ── Read view ─────────────────────────
                                        let date = post.created_at.get(..10).unwrap_or(&post.created_at);
                                        let on_edit = if is_author {
                                            let editing_id = editing_id.clone();
                                            let edit_title = edit_title.clone();
                                            let edit_content = edit_content.clone();
                                            let title = post.title.clone();
                                            let content = post.content.clone();
                                            Some(Callback::from(move |_: MouseEvent| {
                                                editing_id.set(Some(post_id));
                                                edit_title.set(title.clone());
                                                edit_content.set(content.clone());
                                            }))
                                        } else {
                                            None
                                        };
                                        let on_delete = if is_author {
                                            let token = token.clone();
                                            let error = error.clone();
                                            let refresh = refresh.clone();
                                            Some(Callback::from(move |_: MouseEvent| {
                                                let tok = match (*token).clone() {
                                                    Some(t) => t,
                                                    None => { error.set(Some("Not authenticated.".to_owned())); return; }
                                                };
                                                let (error, refresh) = (error.clone(), refresh.clone());
                                                spawn_local(async move {
                                                    match api_delete_post(&tok, post_id).await {
                                                        Ok(_) => refresh.set(*refresh + 1),
                                                        Err(e) => error.set(Some(e)),
                                                    }
                                                });
                                            }))
                                        } else {
                                            None
                                        };
                                        html! {
                                            <div class="post-card" key={post_id}>
                                                <div class="post-header">
                                                    <h3 class="post-title">{ &post.title }</h3>
                                                    <span class="post-meta">
                                                        { format!("by {} · {}", post.author_username, date) }
                                                    </span>
                                                </div>
                                                <p class="post-content">{ &post.content }</p>
                                                if is_author {
                                                    <div class="post-actions">
                                                        <button class="btn btn-sm"
                                                            onclick={on_edit.unwrap()}>
                                                            {"Edit"}
                                                        </button>
                                                        <button class="btn btn-sm btn-danger"
                                                            onclick={on_delete.unwrap()}>
                                                            {"Delete"}
                                                        </button>
                                                    </div>
                                                }
                                            </div>
                                        }
                                    }
                                }) }
                                </div>
                            }
                        </section>
                    </div>
                },

                // ── Login ─────────────────────────────────────────────────────
                View::Login => html! {
                    <div class="auth-container">
                        <div class="auth-card card">
                            <h2>{"Login"}</h2>
                            <form onsubmit={on_login_submit}>
                                <div class="form-group">
                                    <label>{"Username"}</label>
                                    <input ref={login_username_ref.clone()} type="text"
                                        class="form-input" placeholder="Your username" />
                                </div>
                                <div class="form-group">
                                    <label>{"Password"}</label>
                                    <input ref={login_password_ref.clone()} type="password"
                                        class="form-input" placeholder="Your password" />
                                </div>
                                <button type="submit" class="btn btn-full">{"Login"}</button>
                            </form>
                            <p class="auth-switch">
                                {"No account? "}
                                <a onclick={on_go_register.clone()}>{"Register here"}</a>
                            </p>
                        </div>
                    </div>
                },

                // ── Register ──────────────────────────────────────────────────
                View::Register => html! {
                    <div class="auth-container">
                        <div class="auth-card card">
                            <h2>{"Register"}</h2>
                            <form onsubmit={on_register_submit}>
                                <div class="form-group">
                                    <label>{"Username"}</label>
                                    <input ref={reg_username_ref.clone()} type="text"
                                        class="form-input" placeholder="Choose a username" />
                                </div>
                                <div class="form-group">
                                    <label>{"Email"}</label>
                                    <input ref={reg_email_ref.clone()} type="email"
                                        class="form-input" placeholder="you@example.com" />
                                </div>
                                <div class="form-group">
                                    <label>{"Password"}</label>
                                    <input ref={reg_password_ref.clone()} type="password"
                                        class="form-input" placeholder="Choose a password" />
                                </div>
                                <button type="submit" class="btn btn-full">{"Register"}</button>
                            </form>
                            <p class="auth-switch">
                                {"Already have an account? "}
                                <a onclick={on_go_login.clone()}>{"Login here"}</a>
                            </p>
                        </div>
                    </div>
                },
            }}
        </div>
    }
}

// ─── WASM entry point ─────────────────────────────────────────────────────────

/// Called by the browser runtime when the WASM module is loaded.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
