!macro NSIS_HOOK_POSTINSTALL
  CopyFiles /SILENT "$INSTDIR\resources\pthreadVC2.dll" "$INSTDIR\pthreadVC2.dll"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "DNF按键助手"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
    Delete "$INSTDIR\configs\app-config.json"
    RMDir /r "$INSTDIR\configs"
    RMDir /r "$INSTDIR\logs"
  ${EndIf}
!macroend

