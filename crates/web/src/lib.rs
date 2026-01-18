use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <nav>
                    <ul>
                        <li><A href="/">"Dashboard"</A></li>
                        <li><A href="/create/dosage">"New Dose"</A></li>
                        <li><A href="/create/blood-test">"New Blood Test"</A></li>
                        <li><A href="/create/measurement">"New Measurement"</A></li>
                        <li><A href="/view">"View"</A></li>
                        <li><A href="/stats">"Stats"</A></li>
                        <li><A href="/backup">"Backup"</A></li>
                        <li><A href="/calc">"Calculator"</A></li>
                        <li><A href="/vials">"Vials"</A></li>
                        <li><A href="/estrannaise">"Estrannaise"</A></li>
                    </ul>
                </nav>
                <Routes>
                    <Route path="/" view=Dashboard />
                    <Route path="/create/dosage" view=CreateDosage />
                    <Route path="/create/blood-test" view=CreateBloodTest />
                    <Route path="/create/measurement" view=CreateMeasurement />
                    <Route path="/view" view=ViewPage />
                    <Route path="/stats" view=StatsPage />
                    <Route path="/backup" view=BackupPage />
                    <Route path="/calc" view=CalcPage />
                    <Route path="/vials" view=VialsPage />
                    <Route path="/vials/create" view=VialsCreatePage />
                    <Route path="/vials/:id" view=VialsDetailPage />
                    <Route path="/estrannaise" view=EstrannaisePage />
                </Routes>
            </main>
        </Router>
    }
}

fn page_shell(title: &'static str) -> impl IntoView {
    view! {
        <section>
            <h1>{title}</h1>
            <p>"Placeholder for Rust UI rewrite."</p>
        </section>
    }
}

#[component]
fn Dashboard() -> impl IntoView {
    page_shell("Dashboard")
}

#[component]
fn CreateDosage() -> impl IntoView {
    page_shell("Create Dosage")
}

#[component]
fn CreateBloodTest() -> impl IntoView {
    page_shell("Create Blood Test")
}

#[component]
fn CreateMeasurement() -> impl IntoView {
    page_shell("Create Measurement")
}

#[component]
fn ViewPage() -> impl IntoView {
    page_shell("View")
}

#[component]
fn StatsPage() -> impl IntoView {
    page_shell("Stats")
}

#[component]
fn BackupPage() -> impl IntoView {
    page_shell("Backup")
}

#[component]
fn CalcPage() -> impl IntoView {
    page_shell("Calculator")
}

#[component]
fn VialsPage() -> impl IntoView {
    page_shell("Vials")
}

#[component]
fn VialsCreatePage() -> impl IntoView {
    page_shell("Create Vial")
}

#[component]
fn VialsDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|p| p.get("id").cloned().unwrap_or_else(|| "?".into()));
    view! {
        <section>
            <h1>"Vial Detail"</h1>
            <p>"Vial id: "{id}</p>
            <p>"Placeholder for Rust UI rewrite."</p>
        </section>
    }
}

#[component]
fn EstrannaisePage() -> impl IntoView {
    page_shell("Estrannaise")
}

pub fn mount_app() {
    mount_to_body(App);
}
