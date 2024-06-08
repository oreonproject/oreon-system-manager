use std::collections::HashMap;
use std::process::{Command, Stdio};

use gtk::{Box as GBox, Button, Expander, Label, ListBox, Orientation, ScrolledWindow, SearchBar, SearchEntry, Stack, StackSwitcher, Switch};
use gtk::gdk::Display;
use gtk::{prelude::*, Align, CssProvider};
use gtk::{glib, Application, ApplicationWindow};

const APP_ID: &str = "org.oreonproject.SystemManager";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_string(include_str!("style.css"));

        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(begin);

    app.run()
}

/*macro_rules! pkg_list_btn {
    ($a:expr,$e:expr,$($i:expr),+) => {
        let bbox = ListBox::builder()
            .css_classes(["bbox"])
            .build();

        for i in vec![$($i),+] {
            bbox.append(i);
        }

        let b = &Expander::builder()
            .label($e)
            .css_classes(["pkg-ls-btn"])
            .child(&bbox)
            .build();

        $a.append(b);
    }
}*/

macro_rules! pkg_list_btns {
    ($a:expr,$e:expr,$i:expr) => {
        let bbox = ListBox::builder()
            .css_classes(["bbox"])
            .build();

        for i in $i {
            bbox.append(&i);
        }

        let b = &Expander::builder()
            .label($e)
            .css_classes(["pkg-ls-btn"])
            .child(&bbox)
            .build();

        $a.append(b);

    };
}

macro_rules! label {
    () => {
        &Label::new(None)
    };
    ($val:expr) => {
        &Label::new(Some($val))
    };
    ($val:expr,$($css:expr),+) => {
        &Label::builder()
            .label($val)
            .css_classes([$($css),+])
            .build()
    }
}

/*macro_rules! button {
    ($val:expr,$($css:expr),+) => {
        {
            let b = &Button::builder()
                .label($val)
                .css_classes([$($css),+])
                .build();

            b.connect_clicked(|_| {
                Command::new("dnf")
                    .args(["install", $val])
                    .spawn()
                    .expect("Failed to install package");
            });
        }
    }
}
*/
/*macro_rules! pkg_list_item {
    ($a:expr,$e:expr) => {
        $a.append(label!($e,"pkg-ls-item"))
    };
    ($a:expr,$e:expr,$($f:expr),+) => {
        pkg_list_item!($a, $e);
        pkg_list_item!($a, $($f),+);
    }
}*/

macro_rules! switch {
    ($label:expr) => {
        {
            let label = label!($label);
            let opt = Switch::builder()
                .css_classes(["opt"])
                .valign(Align::Start)
                .halign(Align::Fill)
                .build();
            let optbox = GBox::builder()
                .css_classes(["opt-box"])
                .valign(Align::Start)
                .halign(Align::Fill)
                .build();
            optbox.append(label);
            optbox.append(&opt);
            optbox
        }
    }
}

macro_rules! container {
    ($name:expr) => {
        {
            let example_container_button = Button::builder()
                .css_classes(["container-button"])
                .label($name)
                .valign(Align::Start)
                .halign(Align::Start)
                .width_request(200)
                .build();
            example_container_button.connect_clicked(|_| {
                println!("docker pulling {}", $name.to_lowercase());
                let x = std::process::Command::new("pkexec")
                    .args(["docker", "images"])
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to execute child process");
                let y = std::process::Command::new("grep")
                    .arg($name.to_lowercase())
                    .stdin(Stdio::from(x.stdout.unwrap()))
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to execute child process");
                let z: i32 = std::process::Command::new("wc")
                    .arg("-l")
                    .stdin(Stdio::from(y.stdout.unwrap()))
                    .output()
                    .expect("Failed to execute child process")
                    .stdout.iter().map(|x| *x as char).collect::<String>()
                    .trim().parse().unwrap();

                println!("Instances that exist: {z}");

                if z > 0 {
                    std::process::Command::new("kitty")
                        .args(["sudo", "docker", "start", $name.to_lowercase().as_str(), "-i"])
                        .spawn()
                        .expect("failed to execute child process");
                } else {
                    std::process::Command::new("kitty")
                        .args(["sudo", "docker", "run", "--name", $name.to_lowercase().as_str(), "-h", "10-slib", "-e", "LANG=C.UTF-8", "-it", $name.to_lowercase().as_str(), "/bin/bash", "-l"])
                        .spawn()
                        .expect("failed to execute child process");
                }
            });
            example_container_button
        }
    }
}

fn begin(app: &Application) {
    let packages_box = GBox::builder()
        .css_classes(["box", "pkgs"])
        .valign(Align::Center).halign(Align::Fill)
        .hexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let pkgs_search = SearchBar::builder()
        .halign(Align::Fill)
        .valign(Align::Start)
        .css_classes(["search-bar"])
        .build();

    let pkgs_entry = SearchEntry::builder()
        .css_classes(["entries"])
        .halign(Align::Fill)
        .valign(Align::Start)
        .width_request(600)
        .placeholder_text("Search for a package...")
        .sensitive(true)
        .hexpand(true)
        .build();

    let pkg_ls_stack = Stack::builder()
        .css_classes(["pkg-stack"])
        .halign(Align::Fill)
        .valign(Align::Fill)
        .build();

    let pkgs_list = ListBox::builder()
        .css_classes(["pkgs-list"])
        .halign(Align::Center)
        .valign(Align::Center)
        .width_request(600)
        .build();

    let pkgs_cat_list = ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(["pkgs-categories"])
        .halign(Align::Center)
        .valign(Align::Center)
        .width_request(600)
        .build();

    let repo_list_0 = Command::new("dnf")
        .arg("repolist")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to get repo list!");

    let repo_list_1 = Command::new("grep")
        .args(["-Po", "^\\S+"])
        .stdin(repo_list_0.stdout.unwrap())
        .output()
        .expect("Failed to get repo list")
        .stdout.iter().map(|x| *x as char).collect::<String>();

    let repo_list = repo_list_1.split('\n')
        .skip(1);

    let mut pkg_lists = HashMap::new();

    for i in repo_list {
        let x = Command::new("dnf")
                    .args(["repoquery", "--repo", i, "-q", "--qf", "%{name}"])
                    .output()
                    .expect("Failed to fetch package list from repo.")
                    .stdout.iter().map(|x| *x as char).collect::<String>()
                    .clone();

        pkg_lists.insert(i, x.clone().split('\n').map(|x| x.to_string()).collect::<Vec<String>>());
                         // dnf repoquery --repo fedora-cisco-openh264 -q --qf %{name}
    }
 
    /*
    let pkg_list = Command::new("dnf")
        .args(["repoquery", "-q", "--qf", "%{name}"])
        .output()
        .expect("Failed to get package list!")
        .stdout.iter().map(|x| *x as char).collect::<String>();*/

    for (i, x) in pkg_lists.into_iter() {
        let mut list = vec![];

        for y in x {
            list.push(label!(&y).clone());
        }

        pkg_list_btns!(pkgs_cat_list, i, list);
    }

    /*pkg_list_item!(pkgs_list, "Hello, world!", "Goodbye, world!");
    pkg_list_btn!(pkgs_cat_list, "Category 1", label!("Item 1"), label!("Item 2"));
    pkg_list_btn!(pkgs_cat_list, "Category 2", label!("Item 1"), label!("Item 2"));
    pkg_list_btn!(pkgs_cat_list, "Category 3", label!("Item 1"), label!("Item 2"));
    pkg_list_btn!(pkgs_cat_list, "Category 4", label!("Item 1"), label!("Item 2"));*/

    // TODO: Add support for searching.
    pkg_ls_stack.add_titled(&pkgs_list, Some("pkgs-list"), "pkgs-list");
    pkg_ls_stack.add_titled(&pkgs_cat_list, Some("pkgs-cat-list"), "pkgs-cat-list");

    pkg_ls_stack.set_visible_child_name("pkgs-cat-list");

    let pkg_ls_win = ScrolledWindow::builder()
        .halign(Align::Fill)
        .valign(Align::Fill)
        .width_request(600)
        .css_classes(["pkg-ls-win"])
        .height_request(400)
        .build();

    pkg_ls_win.set_child(Some(&pkg_ls_stack));
    
    pkgs_search.connect_entry(&pkgs_entry);

    let text_1 = Label::builder()
        .label("Search for a package or select one from the categories below.")
        .valign(Align::End)
        .halign(Align::Center)
        .css_classes(["label"])
        .build();

    packages_box.append(&pkgs_search);
    packages_box.append(&pkgs_entry);
    packages_box.append(&text_1);
    packages_box.append(&pkg_ls_win);

    let repos_box = GBox::builder()
        .css_classes(["box", "repos"])
        .orientation(Orientation::Vertical)
        .valign(Align::Center).halign(Align::Center)
        .build();

    /*let opt1 = Switch::builder()
        .css_classes(["opt"])
        .valign(Align::Start)
        .halign(Align::Fill)
        .build();*/

    repos_box.append(&switch!("Repo 1"));
    repos_box.append(&switch!("Repo 2"));
    repos_box.append(&switch!("Repo 3"));

    let containers_box = GBox::builder()
        .css_classes(["containers-box"])
        .valign(Align::Fill)
        .halign(Align::Fill)
        .build();

    containers_box.append(&container!("Ubuntu"));
    containers_box.append(&container!("Fedora"));
    containers_box.append(&container!("Debian"));

    let compiler_box = GBox::builder()
        .css_classes(["compiler-box"])
        .valign(Align::Fill)
        .halign(Align::Fill)
        .build();

    let boxes = Stack::builder()
        .valign(Align::Center)
        .halign(Align::Center)
        .height_request(80)
        .css_name("boxes").build();

    boxes.add_titled(&packages_box, Some("Packages"), "Packages");
    boxes.add_titled(&repos_box, Some("Repositories"), "Repositories");
    boxes.add_titled(&containers_box, Some("Containers"), "Containers");
    boxes.add_titled(&compiler_box, Some("Compiler"), "Compiler");

    let sidebar = StackSwitcher::builder()
        .halign(Align::Center)
        .valign(Align::Start)
        .css_classes(["sidebar"])
        .stack(&boxes)
        .orientation(Orientation::Horizontal)
        .build();
 
    let main_box = GBox::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["main"])
        .build();

    main_box.append(&sidebar);
    main_box.append(&boxes);
    
    let main_window = ApplicationWindow::builder()
        .application(app)
        .title("Oreon System Manager")
        .child(&main_box)
        .build();

    main_window.present();
}
