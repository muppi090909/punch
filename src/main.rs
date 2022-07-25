use std::{
    env::current_dir,
    error::Error,
    fs::{self, rename},
    path::{Path, PathBuf},
};
use clap::Parser;
use opener::open;

mod db;
mod in_directory;

#[derive(Debug, Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub struct Args {
    /// to create file
    #[clap(value_parser, multiple_values = true)]
    target: Vec<String>,
    /// to delete
    #[clap(short, long, value_parser)]
    del: Option<Vec<String>>,
    /// creates files inside target directory-first arguement is target
    #[clap(short, long, value_parser, multiple_values = true)]
    r#in: Option<Vec<String>>,
    /// deletes files inside target directory-first arguement is target
    #[clap(long, value_parser, multiple_values = true)]
    din: Option<Vec<String>>,

    /// send the file to trash can
    #[clap(short, long, value_parser, multiple_values = true)]
    trash: Option<Vec<String>>,

    #[clap(short, long)]
    undo: bool,

    #[clap(short, long)]
    show: bool,
    /// Renam a file
    #[clap(short, long, value_parser, multiple_values = true)]
    ren: Option<Vec<String>>,

    /// Move a file from one directory to another.
    #[clap(short, long, multiple_values = true)]
    mve: Option<Vec<String>>,

    /// Lists the files and directories in the current working directory.
    #[clap(short, long)]
    list: bool,
    
    #[clap(short, long, value_parser, multiple_values = true)]
    open: Option<String>,

}

impl Args {
    fn input_type(&self) -> InputType {
        if let Some(_) = self.din {
            return InputType::DeleteIn;
        } else if let Some(_) = self.del {
            return InputType::Del;
        } else if let Some(_) = self.r#in {
            return InputType::CreateIn;
        } else if let Some(_) = self.trash {
            return InputType::Trash;
        } else if self.target.len() > 0 {
            return InputType::Create;
        } else if let true = self.undo {
            return InputType::Undo;
        } else if let true = self.show {
            return InputType::Show;
        } else if let true = self.list {
            return InputType::List;
        } else if let Some(ref args) = self.ren {
            assert!(args.len() == 2, "Expected 2 arguments got {}", args.len());
            return InputType::Rename;
        } else if let Some(_) = self.open {
            return InputType::Open;
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug)]
enum InputType {
    DeleteIn,
    CreateIn,
    Del,
    Create,
    Trash,
    Undo,
    Show,
    Rename,
    Move,
    List,
    Open,
}

struct Trash<'a> {
    trash_path: &'a Path,
}

impl<'a> Trash<'a> {
    fn new(path: &'a Path) -> Self {
        Self { trash_path: path }
    }

    fn copy_recursively(&self, path: &Path) {
        if path.is_dir() {
            let entries = fs::read_dir(path).expect("unable to parse directory");

            fs::create_dir_all(Path::new(self.trash_path).join(path)).unwrap();

            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            // if it is a directory we need to copy the things in the directory . so call again with the new path
                            self.copy_recursively(&path.join(entry.file_name()))
                        } else {
                            fs::copy(
                                path.join(entry.file_name()),
                                Path::new(self.trash_path).join(path.join(entry.file_name())),
                            )
                            .unwrap();
                        }
                    }
                }
            }
        } else {
            fs::copy(path, Path::new(self.trash_path).join(path)).unwrap();
        }
    }
}

fn create_files(args: &Args) {
    let args = args.target.clone();
    for i in 0..args.len() {
        if args[i].contains("/") && args[i].ends_with("/") {
            punch::create_directory(Path::new(&args[i]));
        } else {
            punch::create_file(Path::new(&args[i]));
        }
    }
}
fn delete_files(args: &Args) {
    let args = args.del.clone().unwrap();
    for i in 0..args.len() {
        if args[i].contains("/") && args[i].ends_with("/") {
            punch::remove_directory(Path::new(&args[i])); 
        } else {
            punch::remove_file(Path::new(&args[i])); 
        }
    }
}

fn trash_files(args: &Args) {
    let args = args.trash.clone().unwrap();
    // Check if the .ptrash/ directory exist in ~
    let home_path = match home::home_dir() {
        Some(path) => path,
        _ => panic!("Unable to trash files"),
    };

    let trash_path = home_path.join(Path::new(".ptrash"));
    let trash = Trash::new(&trash_path);

    if !trash.trash_path.exists() {
        // Path Does not Exists
        // Create the Directory
        fs::create_dir(trash.trash_path).expect(format!("error creating trash can").as_str())
    }
    // Move files for directories to crash
    for i in 0..args.len() {
        let file = Path::new(&args[i]);

        trash.copy_recursively(file);
        if Path::new(file).is_dir() {
            //Iterate the directory and move it
            fs::remove_dir_all(file)
                .expect(format!("error removing directory: {:?}", file).as_str());
        } else {
            fs::remove_file(file).expect(format!("error removing directory: {:?}", file).as_str());
        }
    }
}
fn rename_file(args: &Args) {
    let args = args.ren.clone().unwrap();
    let mut source;
    if args[0].clone().starts_with('.') {
        source = current_dir().unwrap()
    } else {
        source = PathBuf::new();
    }
    let mut buf = PathBuf::new();
    args[0].clone().split('/').for_each(|path| {
        if path != "." {
            buf.push(path)
        }
    });
    source = source.join(buf);
    let mut to;
    if args[1].clone().starts_with('.') {
        to = current_dir().unwrap()
    } else {
        to = PathBuf::new();
    }
    let mut buf = PathBuf::new();
    args[1].clone().split('/').for_each(|path| {
        if path != "." {
            buf.push(path)
        }
    });
    to = to.join(buf);
    rename(source, to).expect("Unable to rename");
}

fn move_file(args: &Args) {
    let args = args.mve.clone().unwrap();

    let original_file = Path::new(&args[0]);
    let new_directory = Path::new(&args[1]);

    //number of directories to go back
    let num_to_back = new_directory.to_str().unwrap().parse::<i8>();

    //if second input is a number
    match num_to_back {
        Ok(number) => {
            let mut back_str = String::new();
            //go back a directory for number of times
            for _i in 0..number {
                back_str.push_str("../");
            }
            
            if original_file.exists() {

                fs::File::create(Path::new(&back_str).join(&original_file.file_name().unwrap()))
                    .expect(format!("Failed to create new file: {}", original_file.display()).as_str());

                fs::copy(original_file, Path::new(&back_str).join(&original_file.file_name().unwrap()))
                    .expect(format!("Failed to copy file contents: {}", original_file.display()).as_str());

                fs::remove_file(&original_file)
                    .expect(format!("Failed to delete old file: {}", original_file.display()).as_str());
            }
        },
        Err(_) => {
            if !new_directory.is_dir() {
                println!("Destination directory does not exist, creating new folder.");
                fs::create_dir_all(&new_directory)
                    .expect(format!("Failed to create new directory: ./{}/", new_directory.display()).as_str());
            }
            if original_file.exists(){

                fs::File::create(&new_directory.join(&original_file.file_name().unwrap()))
                    .expect(format!("Failed to create new file: {}", original_file.display()).as_str());

                fs::copy(&original_file, &new_directory.join(original_file.file_name().unwrap()))
                    .expect(format!("Failed to copy file contents: {}", original_file.display()).as_str());

                fs::remove_file(&original_file)
                    .expect(format!("Failed to delete old file: {}", original_file.display()).as_str());
            }
        
        } 
    }
}

fn list_current_directory() {
    let current_dir = std::env::current_dir();
    let paths = fs::read_dir(current_dir.unwrap());
    let paths : Vec<Result<fs::DirEntry, std::io::Error>> = paths.unwrap().collect();

    for path in paths {
        let path = path.unwrap().file_name();
        let path = Path::new(path.to_str().unwrap());
        let mut information = String::new();
        if path.is_dir() {
            information.push_str(&format!("<DIRECTORY>"));
        } else {
            information.push_str(&format!("     <FILE>"));
        }
        println!("{} {}", information, path.to_str().unwrap());
    }
}


fn trash_files(args: &Args) {
    let args = args.trash.clone().unwrap();
    // Check if the .ptrash/ directory exist in ~
    let home_path = match home::home_dir() {
        Some(path) => path,
        _ => panic!("Unable to trash files"),
    };

    let trash_path = home_path.join(Path::new(".punch/trash"));
    let trash = trash::Trash::new(&trash_path);

    if !trash_path.exists() {
        // Path Does not Exists
        // Create the Directory
        fs::create_dir(&trash_path).expect(format!("error creating trash can").as_str())
    }
    // Move files for directories to trash
    // TODO: check if the user has the appropriate permission to move the files
    for i in 0..args.len(){
        let file = Path::new(&args[i]); 
        //TODO: handle trashing files/directories in another directory e.g punch -t test/file1.txt -- This should remove the file
        trash.move_(file); // First Part
        trash.remove_from_source(file); // Second Part
    }
    
#[inline(always)]
fn open_file(args: &Args) -> Result<(), Box<dyn Error>> {
    open(Path::new(args.open.as_ref().unwrap()))?;
    Ok(())

}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    dbg!(args.clone());
    match args.input_type() {

        //order matters for pushing to the database
        /*for files that result increating in current dir
        push to db after for files resulting in a deletion (trash,move,delete,deletein),
        push before*/
        InputType::DeleteIn => {
            db::push(&&args.din.clone().unwrap(), "DeleteIn");
            in_directory::delete_files_dir(&args); 
        },


        InputType::CreateIn => {
            in_directory::create_in_dir(&args);
            db::push(&&args.r#in.clone().unwrap(), "CreateIn")
        }

        InputType::Del => {
            delete_files(&args);
            db::push(&&args.del.clone().unwrap(), "Delete")
        }

        InputType::Create => {
            create_files(&args);
            db::push(&&args.target, "Create")
        }

        InputType::Trash => {
            db::push(&&args.trash.clone().unwrap(), "Trash"); 
            trash_files(&args);
            
        },

        InputType::Undo => db::undo(),

        InputType::Show => db::show(),
        InputType::Rename => rename_file(&args),

        InputType::Move => {
            match &&args.mve.clone().unwrap()[1].parse::<i32>() {
               Ok(_) => (),
               Err(_) => db::push(&&args.mve.clone().unwrap(), "Move"), 
            }  
            move_file(&args)
        },
    }
    Ok(())
}
