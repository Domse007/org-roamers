;;; org-roamers.el --- org-roam enhancements

;; Copyright (C) 2017 Keller Dominik

;; Author: Keller Dominik <example@example.com>
;; URL: https://example.com/package-name.el
;; Version: 0.1-pre
;; Package-Requires: ((emacs "27.1")(org-roam "2.2.2"))
;; Keywords: org-roam org

;; This file is not part of GNU Emacs.

;; This program is free software; you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with this program.  If not, see <http://www.gnu.org/licenses/>.

;;; Commentary:
;; This package tries to enhance org-roam, while also speed up org-roam by
;; omitting the list runtime and do most of the work in rust.

;;;; Installation
;;;;; MELPA
;; If you installed from MELPA, you're done.
;;;;; Git
;; If you installed through git, you first must build the rust core. Install the
;; rust toolchain and for convenience make. Then simply execute *make all* to
;; build all required files.

;;;;; Manual
;; Install these required packages:
;; + org-roam

;; Then put this file in your load-path, and put this in your init
;; file:

;; (require 'package-name)

;;;; Usage
;; Run the following commands:

;;;; Credits
;; This package would not have been possible without the following
;; packages: org-roam[1], org-roam-ui[2].
;;
;;  [1] https://github.com/org-roam/org-roam
;;  [2] https://github.com/org-roam/org-roam-ui

;;; Code:

;;;; Requirements

(require 'org-roam)
(require 'org-roam-node)

(unless (json-available-p)
  (user-error "org-roamers-helm needs JSON support in Emacs;
 please rebuild it using `--with-json'"))

(defgroup org-roamers nil
  "An abstraction layer over org-roam to improve performance."
  :group 'org
  :prefix "org-roamers-")

;;;; Customization

(defcustom org-roamers-url "http://localhost:5000"
  "URL to communicate with the server.")

(defvar org-roamers--last-id ""
  "The last id retrieved by org-roam")

;;;; Functions

(defun org-roamers--emacs-url (id)
  (format "%s/emacs?task=opened&id=%s" org-roamers-url id))

(defun org-roamers-follow ()
  (when (and (org-roam-buffer-p) (buffer-file-name (buffer-base-buffer)))
    (let ((id (org-roam-id-at-point)))
      (when (not (string-equal id org-roamers--last-id))
	(request
	  (org-roamers--emacs-url id)
	  :type "POST"
	  :success
	  (cl-function
	   (lambda (&key data &allow-other-keys)
	     (message "Successfully informed server.")))))
      (setq org-roamers--last-id id))))

(defun org-roamers--buffer-modified (file-name)
  (format "%s/emacs?task=modified&file=%s" org-roamers-url file-name))

(defun org-roamers--save-buffer ()
  (let ((file-name (buffer-file-name (buffer-base-buffer))))
    (when (and (org-roam-buffer-p) file-name)
      (request
	(org-roamers--buffer-modified file-name)
	:type "POST"
	:success
	(cl-function
	 (lambda (&key data &allow-other-keys)
	   (message "Successfully informed server.")))))))

(define-minor-mode org-roamers-mode
  "Enable org-roamers enhances in current buffer."
  :group 'org-roamers
  :global t
  (if org-roamers-mode
      (progn
	(add-hook 'post-command-hook #'org-roamers-follow)
	(add-hook 'after-save-hook #'org-roamers--save-buffer))
    (progn
      (remove-hook 'post-command-hook #'org-roamers-follow)
      (remove-hook 'after-save-hook #'org-roamers--save-buffer))))

(provide 'org-roamers)
;;; org-roamers.el ends here
