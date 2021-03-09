pub mod widget_names {
    pub static dialog_name: &str = "ProgressCopyDlg";
    pub static text_view_copying_total: &str = "TextView_total";
    pub static progress_bar_total: &str = "ProgressBar_total";
    pub static progress_bar_current: &str = "ProgressBar_current";
    pub static suspend_resume_btn: &str = "Suspend_Resume_Btn";
    pub static hideable_cpy_prgrs_br: &str = "hideable_cpy_prgrs_br";
    pub static hideable_cpy_prgrs_br_left_bracket: &str = "left_bracket_hideable";
    pub static hideable_cpy_prgrs_br_right_bracket: &str = "right_bracket_hideable";
    pub static hideable_cpy_button: &str = "hideable_cpy_button";
}
pub mod labels {
    pub fn get_copy_n_items_with_mask_text(is_copy: bool, n_items:usize)->String
    {
        if is_copy{
            format!("Copy {} items with mask:",n_items)
        }
        else
        {
            format!("Move {} items with mask:",n_items)
        }
    } 
    pub fn get_copying_progress_total_background_text(is_copy: bool) -> String {
        if is_copy {
            "Copying...".to_owned()
        } else {
            "Moving...".to_owned()
        }
    }
    pub fn get_copying_progress_total_suspended_background_text(is_copy: bool) -> String {
        if is_copy {
            "Copying paused".to_owned()
        } else {
            "Moving paused".to_owned()
        }
    }
    pub fn get_copy_dialog_title_text(is_copy: bool) -> String {
        if is_copy {
            "Copying".to_owned()
        } else {
            "Moving".to_owned()
        }
    }
    pub fn get_copy_dialog_title_copying_suspended_text(is_copy:bool)->String{
        if is_copy
        {
            "Copying paused".to_owned()
        }
        else
        {
            "Moving paused".to_owned()
        }
    }
    pub fn get_copying_progress_total_text(is_copy: bool) -> String {
        if is_copy {
            "Copying total progress:".to_owned()
        } else {
            "Moving total progress:".to_owned()
        }
    }
    pub fn get_copy_dialog_title(is_copy:bool)->String{
        if is_copy
        {
            "Copy".to_owned()
        }
        else
        {
            "Move".to_owned()
        }
    }
    
    pub fn get_copy_to_text(is_copy:bool)->String{
        if is_copy
        {
            "Copy to:".to_owned()
        }
        else
        {
            "Move to:".to_owned()
        }
    }
}
