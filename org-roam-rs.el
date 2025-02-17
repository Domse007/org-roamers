(require 'org-roam)
(require 'org-roam-node)
(require 'org-roam-utils "./org-roam-rs.so")

(defgroup org-roam-rs nil
  "An abstraction layer over org-roam to improve performance."
  :group 'org
  :prefix "org-roam-rs-")

(defcustom org-roam-rs-num-candidates 10
  "The number of results the db should return to emacs."
  :type 'number
  :group 'org-roam-rs)

;; TODO: is currently not passed to rust
(defcustom org-roam-rs-db-directory
  (if (equal system-type 'windows-nt)
      (let ((dir (getenv "Temp")))
	(if dir dir (error "Could not get temp dir.")))
    "/tmp/org-roam-rs/")
  "The location where the db is stored."
  :type 'string
  :group 'org-roam-rs)

(defun org-roam-rs-init ()
  "Initialize all dbs and prepare system."
  (module-load (expand-file-name "org-roam-rs.so"))
  (org-roam-utils-prepare org-roam-rs-db-directory))

(defun org-roam-rs--get-text (id)
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

(defun org-roam-rs-migrate ()
  (dolist (node (org-roam-node-list))
    (let* ((title (org-roam-node-title node))
	   (id (org-roam-node-id node))
	   ;; TODO: extract body
	   (body (org-roam-rs--get-text id)))
      (org-roam-utils-add-node title id body))))

(defun org-roam-rs--get-candidates (input &optional num-candidates)
  "Get's completion candidates. It returns a list of hash-tabes with fields
title, id, body."
  (let ((json (json-parse-string
	       (org-roam-utils-get-nodes
		input
		(or num-candidates org-roam-rs-num-candidates)))))
    (mapcar (lambda (node) node)
	    (gethash "results" json))))

(define-minor-mode org-roam-rs-mode
  "Enable org-roam-rs enhances in current buffer."
  :group 'org-roam-rs
  (if org-roam-rs-mode
      (push #'org-roam-rs-company company-backends)
    (setq company-backends (remove 'org-roam-rs-company company-backends))))

(defun org-roam-rs-company (command &optional arg &rest rest)
  "Company backend for org-roam-rs."
  (interactive (list 'interactive))
  (cl-case command
    (interactive (company-begin-backend 'org-roam-rs-company))
    (prefix (thing-at-point 'word))
    (candidates (org-roam-rs--get-candidates arg 10))
    (sorted t)
    (no-cache t)))


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;


(require 'helm)

(unless (json-available-p)
  (user-error "org-roam-rs-helm needs JSON support in Emacs;
 please rebuild it using `--with-json'"))

(defun org-roam-rs--helm-get-candidates ()
  (let* ((input helm-pattern)
	 (result (org-roam-rs--get-candidates
		  input
		  org-roam-rs-num-candidates)))
    (mapcar (lambda (node) (gethash "title" node)) result)))

(defun org-roam-rs--helm-format-candidate (node)
  node)

(defun org-roam-rs-helm-node-find ()
  (interactive)
  (org-roam-node-open
   (org-roam-node-from-title-or-alias
    (helm :sources (helm-build-sync-source "Org-roam-rs"
		     :candidates #'org-roam-rs--helm-get-candidates
		     :candidate-transformer #'org-roam-rs--helm-format-candidate
		     :fuzzy-match t
		     :match-dynamic t)
	  :buffer "*helm org-roam-rs*"))))

(provide 'org-roam-rs)
