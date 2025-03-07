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
;; `org-roamers-init': Initialize the package (emacs and rust side)
;; `org-roamers-migrate': To sync the org-roam sqlite db and tantivy.
;;
;; To actually use the package use:
;; `org-roamers-helm-node-find' to find and open a node.

;;;; Credits
;; This package would not have been possible without the following
;; packages: org-roam[1].
;;
;;  [1] https://github.com/org-roam/org-roam

;;; Code:

;;;; Requirements

(require 'org-roam)
(require 'org-roam-node)
(require 'org-roamers-utils (expand-file-name "./org-roamers-utils.so"))

(unless (json-available-p)
  (user-error "org-roamers-helm needs JSON support in Emacs;
 please rebuild it using `--with-json'"))

(defgroup org-roamers nil
  "An abstraction layer over org-roam to improve performance."
  :group 'org
  :prefix "org-roamers-")

;;;; Customization

(defcustom org-roamers-num-candidates 10
  "The number of results the db should return to emacs."
  :type 'number
  :group 'org-roamers)

;; TODO: is currently not passed to rust
(defcustom org-roamers-db-directory
  (if (equal system-type 'windows-nt)
      (let ((dir (getenv "Temp")))
	(if dir dir (error "Could not get temp dir.")))
    "/tmp/org-roamers/")
  "The location where the db is stored."
  :type 'string
  :group 'org-roamers)

(defcustom org-roamers-server-url "localhost:5000"
  "The url where the server is started."
  :type 'string
  :group 'org-roamers)

;;;; Functions

;;;###autoload
(defun org-roamers-init ()
  "Initialize all dbs and prepare system."
  ;; (module-load (expand-file-name "org-roamers-utils.so"))
  (org-roamers-utils-prepare org-roamers-db-directory
			     org-roam-db-location))

(defun org-roamers--get-text (id)
  "Retrieve the text from org-node ID.
This is stolen from org-roam-ui."
  (let* ((node (org-roam-populate (org-roam-node-create :id id)))
	 (file (org-roam-node-file node)))
    (org-roam-with-temp-buffer file
      (when (> (org-roam-node-level node) 0)
        ;; Heading nodes have level 1 and greater.
        (goto-char (org-roam-node-point node))
        (org-narrow-to-element))
      (buffer-substring-no-properties (buffer-end -1) (buffer-end 1)))))

(defun org-roamers-migrate ()
  (dolist (node (org-roam-node-list))
    (let* ((title (org-roam-node-title node))
	   (id (org-roam-node-id node))
	   (file (org-roam-node-file node))
	   ;; TODO: extract body
	   (body (org-roamers--get-text id)))
      (org-roamers-utils-add-node title id body file))))

(defun org-roamers--get-candidates (input &optional num-candidates)
  "Get's completion candidates. It returns a list of hash-tabes with fields
title, id, body."
  (let ((json (json-parse-string
	       (org-roamers-utils-get-nodes
		input
		(or num-candidates org-roamers-num-candidates)))))
    (mapcar (lambda (node) node)
	    (gethash "results" json))))

(define-minor-mode org-roamers-mode
  "Enable org-roamers enhances in current buffer."
  :group 'org-roamers
  (if org-roamers-mode
      (push #'org-roamers-company company-backends)
    (setq company-backends (remove 'org-roamers-company company-backends))))

(defun org-roamers-company (command &optional arg &rest rest)
  "Company backend for org-roamers."
  (interactive (list 'interactive))
  (cl-case command
    (interactive (company-begin-backend 'org-roamers-company))
    (prefix (list (company-grab-word) (company-grab-word-suffix)))
    (candidates (let ((candidates (org-roamers--get-candidates arg 10)))
		  (message "Got %s candidates with %s." (length candidates) arg)

		  (if (string-empty-p arg)
		      candidates
		    (company-substitute-prefix
		     arg
		     (all-completions arg candidates)))

		  ))
    (ignore-case t)
    (sorted t)
    (no-cache t)))

(defun org-roamers--completing-read-get-candidates (input _func _flag)
  (let* ((result (org-roamers--get-candidates
		  input
		  org-roamers-num-candidates)))
    (message "num res: %s" (length result))
    (mapcar (lambda (node) (gethash "title" node)) result)))

(defun org-roamers-node-find ()
  "A faster more flexible alternative to `org-roam-node-find'"
  (interactive)
  (completing-read "Node: "
		   #'org-roamers--completing-read-get-candidates))


;; (defun org-roamers-helm-node-find ()
;;   (interactive)
;;   (org-roam-node-open
;;    (org-roam-node-from-title-or-alias
;;     (helm :sources (helm-build-sync-source "Org-roamers"
;; 		     :candidates #'org-roamers--helm-get-candidates
;; 		     :candidate-transformer #'org-roamers--helm-format-candidate
;; 		     :fuzzy-match t
;; 		     :match-dynamic t)
;; 	  :buffer "*helm org-roamers*"))))

;;; Section server

(defconst org-roamers-server-root-dir
  (expand-file-name "web/" default-directory)
  "Path to web directory relative to this file.")

(defun org-roamers-server-start ()
  (interactive)
  (org-roamers-utils-server-start-server org-roamers-server-url
					 org-roamers-server-root-dir))

(provide 'org-roamers)
;;; org-roamers.el ends here
