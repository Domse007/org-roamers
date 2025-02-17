(require 'org-roam)
(require 'org-roam-node)

(defgroup org-roam-rs nil
  "An abstraction layer over org-roam to improve performance."
  :group 'org
  :prefix "org-roam-rs-")

(defcustom org-roam-rs-num-candidates 10
  "The number of results the db should return to emacs."
  :type 'number
  :group 'org-roam-rs)

;; TODO:
(module-load "/home/dominik/Code/Web/org-roam-rs/target/debug/liborg_roam_rs.so")

;; TODO: is currently not passed to rust
(defcustom org-roam-rs-db-directory
  (if (equal system-type 'windows-nt)
      (let ((dir (getenv "Temp")))
	(if dir dir (error "Could not get temp dir.")))
    "/tmp/org-roam-rs/")
  "The location where the db is stored."
  :type 'string
  :group 'org-roam-rs)

(defun org-roam-rs-migrate ()
  (dolist (node (org-roam-node-list))
    (let ((title (org-roam-node-title node))
	  (id (org-roam-node-id node))
	  ;; TODO: extract body
	  (body ""))
      (org-roam-utils-add-node title id body))))


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;


(require 'helm)

(unless (json-available-p)
  (user-error "org-roam-rs-helm needs JSON support in Emacs;
 please rebuild it using `--with-json'"))

(defun org-roam-rs--helm-get-candidates ()
  (let* ((input helm-pattern)
	 (json (json-parse-string (org-roam-utils-get-nodes
				   input
				   org-roam-rs-num-candidates))))
    (mapcar (lambda (node) (gethash "title" node))
	    (gethash "results" json))))

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
