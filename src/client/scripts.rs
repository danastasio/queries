use gtk4::prelude::*;
use gtk4::*;
use crate::React;
use crate::ui::{QueriesEditor, ScriptList, SaveDialog, OpenDialog};
use crate::ui::MainMenu;
use crate::Callbacks;
use crate::ui::FileList;
use std::boxed;
use std::thread;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::ui::QueriesWindow;

pub enum ScriptAction {

    OpenRequest(String),

    OpenSuccess(OpenedFile),

    OpenFailure(String),

    // File position and whether the request is "forced" (i.e. asks for user confirmation).
    CloseRequest(usize, bool),

    CloseConfirm(usize),

    SaveRequest(usize),

    Opened(String),

    Closed(String),

    NewRequest,

    ActiveTextChanged(Option<String>),

    WindowCloseRequest(ApplicationWindow),

    SetSaved(usize, bool),

    Select(Option<usize>)

}

pub struct OpenedScripts {

    send : glib::Sender<ScriptAction>,

    on_open : Callbacks<OpenedFile>,

    on_save : Callbacks<OpenedFile>,

    on_file_changed : Callbacks<usize>,

    on_file_persisted : Callbacks<usize>,

    on_active_text_changed : Callbacks<Option<String>>,

    on_new : Callbacks<OpenedFile>,

    on_closed : Callbacks<(usize, usize)>,

    on_close_confirm : Callbacks<(OpenedFile)>,

    on_selected : Callbacks<Option<usize>>

}

impl OpenedScripts {

    pub fn new() -> Self {
        let (send, recv) = glib::MainContext::channel::<ScriptAction>(glib::PRIORITY_DEFAULT);
        let on_open : Callbacks<OpenedFile> = Default::default();
        let on_new : Callbacks<OpenedFile> = Default::default();
        let on_save : Callbacks<OpenedFile> = Default::default();
        let on_file_changed : Callbacks<usize> = Default::default();
        let on_file_persisted : Callbacks<usize> = Default::default();
        let on_selected : Callbacks<Option<usize>> = Default::default();
        let on_closed : Callbacks<(usize, usize)> = Default::default();
        let on_active_text_changed : Callbacks<Option<String>> = Default::default();
        let on_close_confirm : Callbacks<OpenedFile> = Default::default();
        let mut files : Vec<OpenedFile> = Vec::new();
        let mut selected : Option<usize> = None;

        let mut app_win : Option<ApplicationWindow> = None;

        recv.attach(None, {
            let send = send.clone();
            let (on_open, on_new, on_save, on_selected, on_closed, on_close_confirm, on_file_changed, on_file_persisted) = (
                on_open.clone(),
                on_new.clone(),
                on_save.clone(),
                on_selected.clone(),
                on_closed.clone(),
                on_close_confirm.clone(),
                on_file_changed.clone(),
                on_file_persisted.clone()
            );
            let (on_active_text_changed) = (on_active_text_changed.clone());
            move |action| {
                match action {
                    ScriptAction::NewRequest => {
                        if files.len() == 16 {
                            return Continue(true);
                        }
                        let n = files.len();
                        let new_file = OpenedFile { path : None, name : format!("Untitled {}.sql", files.len() + 1), saved : true, content : None, index : files.len() };
                        files.push(new_file.clone());
                        println!("{:?}", files);
                        on_new.borrow().iter().for_each(|f| f(new_file.clone()) );
                    },
                    ScriptAction::OpenRequest(path) => {
                        if files.len() == 16 {
                            return Continue(true);
                        }
                        thread::spawn({
                            let send = send.clone();
                            move || {
                                // Open file and write content to OpenedFile.
                                // Send OpenSuccess event
                            }
                        });
                    },
                    ScriptAction::CloseRequest(ix, force) => {
                        if force {
                            files.remove(ix);
                            let n = files.len();
                            on_closed.borrow().iter().for_each(|f| f((ix, n)) );
                            println!("File closed");
                            if let Some(win) = &app_win {
                                win.destroy();
                            }
                        } else {
                            if files[ix].saved {
                                files.remove(ix);
                                let n = files.len();
                                on_closed.borrow().iter().for_each(|f| f((ix, n)) );
                            } else {
                                on_close_confirm.borrow().iter().for_each(|f| f(files[ix].clone()) );
                            }
                        }
                    },
                    ScriptAction::SetSaved(ix, saved) => {
                        files[ix].saved = saved;
                        if saved {
                            on_file_persisted.borrow().iter().for_each(|f| f(ix) );
                        } else {
                            on_file_changed.borrow().iter().for_each(|f| f(ix) );
                        }
                    },
                    ScriptAction::OpenSuccess(file) => {
                        on_open.borrow().iter().for_each(|f| f(file.clone()) );
                    },
                    ScriptAction::Select(opt_ix) => {
                        selected = opt_ix;
                        on_selected.borrow().iter().for_each(|f| f(opt_ix) );
                    },
                    ScriptAction::WindowCloseRequest(win) => {
                        if let Some(file) = files.iter().filter(|file| !file.saved ).next() {
                            on_close_confirm.borrow().iter().for_each(|f| f(file.clone()) );
                            app_win = Some(win);
                        } else {
                            win.destroy();
                        }
                    },
                    _ => { }
                }
                Continue(true)
            }
        });
        Self { on_open, on_save, on_new, send, on_selected, on_closed, on_close_confirm, on_file_changed, on_file_persisted, on_active_text_changed }
    }

    pub fn connect_new<F>(&self, f : F)
    where
        F : Fn(OpenedFile) + 'static
    {
        self.on_new.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_selected<F>(&self, f : F)
    where
        F : Fn(Option<usize>) + 'static
    {
        self.on_selected.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_opened<F>(&self, f : F)
    where
        F : Fn(OpenedFile) + 'static
    {
        self.on_open.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_closed<F>(&self, f : F)
    where
        F : Fn((usize, usize)) + 'static
    {
        self.on_closed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_close_confirm<F>(&self, f : F)
    where
        F : Fn(OpenedFile) + 'static
    {
        self.on_close_confirm.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_file_changed<F>(&self, f : F)
    where
        F : Fn(usize) + 'static
    {
        self.on_file_changed.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_file_persisted<F>(&self, f : F)
    where
        F : Fn(usize) + 'static
    {
        self.on_file_persisted.borrow_mut().push(boxed::Box::new(f));
    }

    pub fn connect_on_active_text_changed<F>(&self, f : F)
    where
        F : Fn(Option<String>) + 'static
    {
        self.on_active_text_changed.borrow_mut().push(boxed::Box::new(f));
    }

}

// To save file...
/*if let Some(path) = file.path {
        if Self::save_file(&path, self.get_text()) {
            self.file_list.mark_current_saved();
            println!("Content written into file");
        } else {
            println!("Unable to save file");
        }
    } else {
        self.sql_save_dialog.set_filename(&file.name);
        self.sql_save_dialog.run();
        self.sql_save_dialog.hide();
    }
}
*/

// TO open file..
// view.get_buffer().map(|buf| buf.set_text(&content) );

// To get text...
/*
pub fn get_text(&self) -> String {
    if let Some(buffer) = self.view.borrow().get_buffer() {
        let txt = buffer.get_text(
            &buffer.get_start_iter(),
            &buffer.get_end_iter(),
            true
        ).unwrap();
        txt.to_string()
    } else {
        panic!("Unable to retrieve text buffer");
    }
} */

fn save_file(path : &Path, content : String) -> bool {
    if let Ok(mut f) = File::create(path) {
        if f.write_all(content.as_bytes()).is_ok() {
            println!("Content written to file");
            true
        } else {
            false
        }
    } else {
        println!("Unable to write into file");
        false
    }
}

#[derive(Debug, Clone)]
pub struct OpenedFile {
    pub name : String,
    pub path : Option<String>,
    pub content : Option<String>,
    pub saved : bool,
    pub index : usize
}

pub struct ScriptHistory {

}

impl ScriptHistory {

    pub fn new() -> Self {

        Self { }
    }

}

impl React<SaveDialog> for OpenedScripts {

    fn react(&self, dialog : &SaveDialog) {
        // call on_saved
    }

}

impl React<MainMenu> for OpenedScripts {

    fn react(&self, menu : &MainMenu) {
        let send = self.send.clone();
        menu.action_new.connect_activate(move |_,_| {
            send.send(ScriptAction::NewRequest);
        });
    }

}

impl React<ScriptList> for OpenedScripts {

    fn react(&self, list : &ScriptList) {
        let send = self.send.clone();
        list.new_btn.connect_clicked(move |_| {
            send.send(ScriptAction::NewRequest);
        });
    }
}

impl React<OpenDialog> for OpenedScripts {

    fn react(&self, dialog : &OpenDialog) {
        let send = self.send.clone();
        dialog.dialog.connect_response(move |dialog, resp| {
            match resp {
                ResponseType::Accept => {
                    if let Some(path) = dialog.file().and_then(|f| f.path() ) {
                        send.send(ScriptAction::OpenRequest(path.to_str().unwrap().to_string()));
                    }
                },
                _ => { }
            }
        });
    }

}

impl React<FileList> for OpenedScripts {

    fn react(&self, list : &FileList) {
        list.list.connect_row_selected({
            let send = self.send.clone();
            move |_, opt_row| {
                if let Some(row) = opt_row {
                    send.send(ScriptAction::Select(Some(row.index() as usize)));
                } else {
                    send.send(ScriptAction::Select(None));
                }
            }
        });
        list.close_action.connect_activate({
            let send = self.send.clone();
            move |action, param| {
                let ix = param.unwrap().get::<i32>().unwrap();
                send.send(ScriptAction::CloseRequest(ix as usize, false));
            }
        });
    }

}

impl React<QueriesEditor> for OpenedScripts {

    fn react(&self, editor : &QueriesEditor) {
        editor.views.iter().enumerate().for_each(|(ix, view)| {
            let send = self.send.clone();
            view.buffer().connect_changed(move |_| {
                send.send(ScriptAction::SetSaved(ix, false));
            });
        });
        editor.ignore_file_save_action.connect_activate({
            let send = self.send.clone();
            move |action, param| {
                if let Some(variant) = param {
                    let ix = variant.get::<i32>().unwrap();
                    if ix >= 0 {
                        send.send(ScriptAction::CloseRequest(ix as usize, true));
                    } else {
                        panic!("Ix is nonzero");
                    }
                } else {
                    panic!("Action does not have parameter");
                }
            }
        });
    }

}

impl React<QueriesWindow> for OpenedScripts {

    fn react(&self, win : &QueriesWindow) {
        let send = self.send.clone();
        win.window.connect_close_request(move |win| {
            send.send(ScriptAction::WindowCloseRequest(win.clone()));
            glib::signal::Inhibit(true)
        });
    }
}

