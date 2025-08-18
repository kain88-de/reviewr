use crate::core::models::{DataPath, Employee, validate_employee_name};
use fs4::FileExt;
use log::{info, warn};
use std::fs;
use std::io::{self, Write};

pub struct EmployeeService;

impl EmployeeService {
    pub fn add_employee(data_path: &DataPath, employee_name: &str) -> io::Result<()> {
        validate_employee_name(employee_name)?;

        println!("Adding new employee: {employee_name}");
        print!("Title: ");
        io::stdout().flush()?;
        let mut title = String::new();
        io::stdin().read_line(&mut title)?;

        print!("Committer Email (optional): ");
        io::stdout().flush()?;
        let mut email = String::new();
        io::stdin().read_line(&mut email)?;
        let email = email.trim();
        let email = if email.is_empty() {
            None
        } else {
            Some(email.to_string())
        };

        Self::add_employee_with_data(data_path, employee_name, title.trim(), email)
    }

    pub fn add_employee_with_data(
        data_path: &DataPath,
        employee_name: &str,
        title: &str,
        committer_email: Option<String>,
    ) -> io::Result<()> {
        validate_employee_name(employee_name)?;

        if title.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Title cannot be empty",
            ));
        }

        let employee = Employee {
            name: employee_name.to_string(),
            title: title.to_string(),
            committer_email,
        };

        let toml = toml::to_string(&employee).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize employee data: {e}"),
            )
        })?;

        let path = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));

        // Use file locking to prevent concurrent modifications
        let file = fs::File::create(&path)?;
        FileExt::lock_exclusive(&file)?;
        fs::write(&path, toml)?;
        FileExt::unlock(&file)?;

        info!("Employee '{}' added to {}", employee_name, path.display());
        println!("Employee '{employee_name}' added.");
        Ok(())
    }

    pub fn employee_exists(data_path: &DataPath, employee_name: &str) -> bool {
        let employee_file = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));
        employee_file.exists()
    }

    pub fn list_employees(data_path: &DataPath) -> io::Result<Vec<String>> {
        let mut employees = Vec::new();

        if !data_path.employees_dir.exists() {
            return Ok(employees);
        }

        for entry in fs::read_dir(&data_path.employees_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                employees.push(name.to_string());
            }
        }

        employees.sort();
        Ok(employees)
    }

    pub fn get_employee(data_path: &DataPath, employee_name: &str) -> io::Result<Employee> {
        validate_employee_name(employee_name)?;

        let employee_file = data_path
            .employees_dir
            .join(format!("{employee_name}.toml"));

        // Use file locking for read operations too
        let file = fs::File::open(&employee_file)?;
        FileExt::lock_shared(&file)?;
        let content = fs::read_to_string(&employee_file)?;
        FileExt::unlock(&file)?;

        let employee: Employee = toml::from_str(&content).map_err(|e| {
            warn!(
                "Failed to parse employee file {}: {}",
                employee_file.display(),
                e
            );
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid employee file format: {e}"),
            )
        })?;
        Ok(employee)
    }

    pub fn update_employee(
        data_path: &DataPath,
        old_name: &str,
        new_name: &str,
        title: &str,
        committer_email: Option<String>,
    ) -> io::Result<()> {
        validate_employee_name(old_name)?;
        validate_employee_name(new_name)?;

        if title.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Title cannot be empty",
            ));
        }

        let old_path = data_path.employees_dir.join(format!("{old_name}.toml"));
        let new_path = data_path.employees_dir.join(format!("{new_name}.toml"));

        // Create updated employee
        let employee = Employee {
            name: new_name.to_string(),
            title: title.to_string(),
            committer_email,
        };

        let toml = toml::to_string(&employee).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize employee data: {e}"),
            )
        })?;

        // Use file locking when writing
        let file = fs::File::create(&new_path)?;
        FileExt::lock_exclusive(&file)?;
        fs::write(&new_path, toml)?;
        FileExt::unlock(&file)?;

        // Remove old file if name changed
        if old_name != new_name && old_path.exists() {
            fs::remove_file(&old_path)?;
            info!("Removed old employee file: {}", old_path.display());
        }

        info!("Employee '{new_name}' updated (was '{old_name}')");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_add_employee_with_data() {
        let temp_dir = tempdir().unwrap();
        let data_path = DataPath::new(Some(temp_dir.path().to_path_buf())).unwrap();
        fs::create_dir_all(&data_path.employees_dir).unwrap();

        EmployeeService::add_employee_with_data(&data_path, "John Doe", "Engineer", None).unwrap();

        let employee = EmployeeService::get_employee(&data_path, "John Doe").unwrap();
        assert_eq!(employee.name, "John Doe");
        assert_eq!(employee.title, "Engineer");
    }

    #[test]
    fn test_update_employee() {
        let temp_dir = tempdir().unwrap();
        let data_path = DataPath::new(Some(temp_dir.path().to_path_buf())).unwrap();
        fs::create_dir_all(&data_path.employees_dir).unwrap();

        // Create initial employee
        EmployeeService::add_employee_with_data(&data_path, "John Doe", "Engineer", None).unwrap();

        // Update employee
        EmployeeService::update_employee(
            &data_path,
            "John Doe",
            "John Smith",
            "Senior Engineer",
            None,
        )
        .unwrap();

        // Verify old file is gone and new one exists
        assert!(!EmployeeService::employee_exists(&data_path, "John Doe"));
        assert!(EmployeeService::employee_exists(&data_path, "John Smith"));

        let employee = EmployeeService::get_employee(&data_path, "John Smith").unwrap();
        assert_eq!(employee.name, "John Smith");
        assert_eq!(employee.title, "Senior Engineer");
    }

    #[test]
    fn test_update_employee_same_name() {
        let temp_dir = tempdir().unwrap();
        let data_path = DataPath::new(Some(temp_dir.path().to_path_buf())).unwrap();
        fs::create_dir_all(&data_path.employees_dir).unwrap();

        // Create initial employee
        EmployeeService::add_employee_with_data(&data_path, "John Doe", "Engineer", None).unwrap();

        // Update title only
        EmployeeService::update_employee(
            &data_path,
            "John Doe",
            "John Doe",
            "Senior Engineer",
            None,
        )
        .unwrap();

        let employee = EmployeeService::get_employee(&data_path, "John Doe").unwrap();
        assert_eq!(employee.name, "John Doe");
        assert_eq!(employee.title, "Senior Engineer");
    }
}
