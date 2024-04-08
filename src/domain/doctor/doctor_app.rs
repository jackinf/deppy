pub trait DoctorApp {
    async fn check(&self, project: &str);
}

pub struct DoctorAppImpl;
impl DoctorApp for DoctorAppImpl {
    async fn check(&self, project: &str) {
        println!("Checking project: {}", project);
    }
}
