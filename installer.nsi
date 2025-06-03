Outfile "org-roamers-gui-setup.exe"
InstallDir "$PROGRAMFILES\Org Roamers GUI"
RequestExecutionLevel admin

Section "Install"
  ; Output directory on user's system
  SetOutPath "$INSTDIR"

  ; Main binary
  File "target\release\org-roamers-gui.exe"

  File /r "server_conf.json"

  File /r "web\public\org-roamers-gui.png"

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Shortcut on Desktop
  CreateShortCut "$DESKTOP\Org Roamers GUI.lnk" "$INSTDIR\org-roamers-gui.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\org-roamers-gui.exe"
  Delete "$INSTDIR\server_conf.json"
  Delete "$INSTDIR\web\public\org-roamers-gui.png"
  RMDir "$INSTDIR"
  Delete "$DESKTOP\Org Roamers GUI.lnk"
SectionEnd
