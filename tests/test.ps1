$global:VEN_LAST_DIR = $null

function global:__ven_activate {
    $current_dir = $PWD.Path
    if ($global:VEN_LAST_DIR -eq $current_dir) { return }
    $global:VEN_LAST_DIR = $current_dir
    echo "Directory changed to $current_dir"
}

if (-not $global:__ven_prompt_hooked) {
    $global:__ven_prompt_hooked = $true
    $global:__ven_old_prompt = ${function:prompt}
    
    function global:prompt {
        __ven_activate
        if ($global:__ven_old_prompt) {
            & $global:__ven_old_prompt
        } else {
            "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
        }
    }
}
