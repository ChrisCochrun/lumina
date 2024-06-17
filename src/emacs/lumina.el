;;; lumina.el -*- lexical-binding: t; -*-

(require 'cl-lib)
(require 'request)

(defvar lumina-db (sqlite-open "~/.local/share/lumina/library-db.sqlite3"))

(defvar lumina-songs (sqlite-select lumina-db "select * from songs;"))

(defvar lumina-songs-set (sqlite-select lumina-db "select * from songs;" nil :set))

(defvar lumina-buffer)

(define-derived-mode lumina-mode tabulated-list-mode "lumina:songs"
  "A mode for working with the lumina database and updating and adding songs"
  (setf tabulated-list-format ['("Title" 10 t)
                               '("Lyrics" 30 nil)
                               '("Author" 10 t)
                               '("CCLI" 7 nil)
                               '("B" 1 nil)
                               '("BT" 10 nil)
                               '("h" 1 nil)
                               '("v" 1 nil)
                               '("f" 10 nil)
                               '("fs" 2 nil)]))

(defun lumina-get-ids ()
  "Gets the ids of all songs in the sql result"
  (cl-loop for song in lumina-songs
           collect (car song)))

(defun lumina-get-songs ()
  "Gets the songs from the sql result without the id"
  (cl-loop for song in lumina-songs
           collect (seq--into-vector (cdr song))))

(defun lumina-table ()
  "creates the list necessary for the tabulated-list-entries table")

(cl-mapcar #'cons (lumina-get-ids) (lumina-get-songs))

(defun lumina ()
  "Start lumina up"
  (interactive)
  (setf lumina-buffer (get-buffer-create "*lumina*"))
  (with-current-buffer lumina-buffer
    (lumina-mode)
    (setf tabulated-list-entries
          (cl-mapcar #'list (lumina-get-ids) (lumina-get-songs)))
    (tabulated-list-print t t))
  (switch-to-buffer lumina-buffer))

;;Elements are in this order, id, title, lyrics, author,
;;ccli, audio file, verse order, background, background type,
;;halign, valign, font, fontsize

(defun lumina-select-song ()
  "Select which song to edit"
  (interactive)
  (with-current-buffer (get-buffer-create "*lumina*")
    (org-mode)
    (delete-region (point-min) (point-max))
    (point-min)
    (let* ((title (completing-read "Select a song: " (cl-loop for song in lumina-songs
                                                               collect (cadr song))))
           (song (cl-loop for song in lumina-songs
                          when (string= (cadr song) title)
                          return song))
           (lyrics (elt song 2))
           (author (elt song 3))
           (ccli (elt song 4))
           (audio (elt song 5))
           (verse-order (elt song 6))
           (background (elt song 7))
           (background-type (elt song 8))
           (halign (elt song 9))
           (valign (elt song 10))
           (font (elt song 11))
           (font-size (elt song 12)))
      (insert (concat "#+TITLE: " title))
      (newline)
      (insert (concat "#+AUTHOR: " author))
      (newline)
      (insert (concat "#+AUDIO: " audio))
      (newline)
      (insert (concat "#+VERSE_ORDER: " verse-order))
      (newline)
      (insert (concat "#+BACKGROUND: " background))
      (newline)
      (insert (concat "#+BACKGROUND_TYPE: " background-type))
      (newline)
      (insert (concat "#+HALIGN: " halign))
      (newline)
      (insert (concat "#+VALIGN: " valign))
      (newline)
      (insert (concat "#+FONT: " font))
      (newline)
      (insert (concat "#+FONT_SIZE: " (number-to-string font-size)))
      (newline)
      (newline)
      (insert (concat "* Lyrics\n" lyrics))
      (print song)
      (setf lumina-current-song title)))
  (switch-to-buffer "*lumina*"))

(defvar lumina-current-song)

(defun lumina-grab-lyrics ()
  (with-current-buffer (get-buffer "*lumina*")
    (goto-char (point-min))
    (search-forward "* Lyrics")
    (next-line)
    (move-to-left-margin)
    (buffer-substring-no-properties (point) (point-max))))

(defun lumina-get-verse-order ()
  (with-current-buffer (get-buffer "*lumina*")
    (goto-char (point-min))
    (search-forward "#+VERSE_ORDER: ")
    (buffer-substring-no-properties (point) nil)))

(defvar lumina-lyrics-update-query
  (concat "update songs set lyrics = \"?"
          (lumina-grab-lyrics)
          "\" where title = "
          lumina-current-song))

(defun lumina-update-lyrics ()
  "Update the lyrics in the db"
  (interactive)
  (sqlite-execute lumina-db
                  "update songs set lyrics = ? where title = ?"
                  `(,(lumina-grab-lyrics) ,lumina-current-song))
  (setf lumina-songs (sqlite-select lumina-db "select * from songs;")))


(defun lumina-get-lyrics-genius (song)
  "retrieve lyrics to a song from genius lyrics"
  (let* ((url (concat "https://api.genius.com/search?"
                      "access_token="
                      "R0Y0ZW50Il9LSh5su3LKfdyfmQRx41NpVvLFJ0VxMo-hQ_4H1OVg_IE0Q-UUoFQx"
                      "&q="
                      song))
         (songs 
          (cl-loop for song across
                   (cdr (cadadr
                         (plz 'get (url-encode-url url) :as #'json-read)))
                   collect `(,(concat
                               (cdr (elt (elt song 3) 19)) " by "
                               (cdr (elt (elt (elt song 3) 23) 7))
                               " with id "
                               (number-to-string
                                (cdr (elt (elt song 3) 7)))))))
         (selected-song (completing-read "song? " songs))
         (id (replace-regexp-in-string "[^0-9]" "" selected-song)))
    (plz 'get
      (url-encode-url
       (concat "https://api.genius.com/songs/" id
        "?access_token=R0Y0ZW50Il9LSh5su3LKfdyfmQRx41NpVvLFJ0VxMo-hQ_4H1OVg_IE0Q-UUoFQx")))))

(defun lumina-presentation ()
  "Creates a lumina presentation from an org document")

(defun lumina-add-item ()
  "add and item to the lumina presentation org buffer"
  (interactive)
  (with-current-buffer (get-buffer "*lumina*")
    (goto-char (point-max))
    (insert "
* 
:PROPERTIES:
:TITLE: 
:AUTHOR: 
:AUDIO: 
:VERSE_ORDER: 
:BACKGROUND: 
:BACKGROUND_TYPE: 
:HALIGN: 
:VALIGN: 
:FONT: 
:FONT_SIZE: 
:END:

")))

(cdr (elt (elt (elt (lumina-get-lyrics-genius "Death Was Arrested") 0) 3) 7))

(lumina-get-lyrics-genius "Death Was Arrested")
