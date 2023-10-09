#include "songsqlmodel.h"

#include <QDateTime>
#include <QDebug>
#include <QSqlError>
#include <QSqlRecord>
#include <QSqlQuery>
#include <QSql>
#include <QSqlDatabase>
#include <QItemSelectionModel>
#include <qabstractitemmodel.h>
#include <qdebug.h>
#include <qglobal.h>
#include <qobjectdefs.h>
#include <qregexp.h>
#include <qsqlquery.h>
#include <qsqlrecord.h>

static const char *songsTableName = "songs";

static void createTable()
{
  if(QSqlDatabase::database().tables().contains(songsTableName)) {
    return;
  }

  QSqlQuery query;
  if (!query.exec("CREATE TABLE IF NOT EXISTS 'songs' ("
                  "  'id' INTEGER NOT NULL,"
                  "  'title' TEXT NOT NULL,"
                  "  'lyrics' TEXT,"
                  "  'author' TEXT,"
                  "  'ccli' TEXT,"
                  "  'audio' TEXT,"
                  "  'vorder' TEXT,"
                  "  'background' TEXT,"
                  "  'backgroundType' TEXT,"
                  "  'horizontalTextAlignment' TEXT,"
                  "  'verticalTextAlignment' TEXT,"
                  "  'font' TEXT,"
                  "  'fontSize' INTEGER,"
                  "  PRIMARY KEY(id))")) {
    qFatal("Failed to query database: %s",
           qPrintable(query.lastError().text()));
  }
  // qDebug() << query.lastQuery();
  // qDebug() << "inserting into songs";

  query.exec(
      "INSERT INTO songs (title, lyrics, author, ccli, audio, vorder, "
      "background, backgroundType, horizontalTextAlignment, verticalTextAlignment, font, fontSize) VALUES ('10,000 Reasons', '10,000 reasons "
      "for my heart to sing', 'Matt Redman', '13470183', '', '', '', '', 'center', 'center', '', '')");
  // qDebug() << query.lastQuery();
  query.exec("INSERT INTO songs (title, lyrics, author, ccli, audio, vorder, "
             "background, backgroundType, horizontalTextAlignment, verticalTextAlignment, font, fontSize) VALUES ('River', 'Im going down to "
             "the river', 'Jordan Feliz', '13470183', '', '', '', '', 'center', 'center', '', '')");
  query.exec(
      "INSERT INTO songs (title, lyrics, author, ccli, audio, vorder, "
      "background, backgroundType, horizontalTextAlignment, verticalTextAlignment, font, fontSize) VALUES ('Marvelous Light', 'Into marvelous "
      "light Im running', 'Chris Tomlin', '13470183', '', '', '', '', 'center', 'center', '', '')");

  // qDebug() << query.lastQuery();
  query.exec("select * from songs");
  // qDebug() << query.lastQuery();
}

SongSqlModel::SongSqlModel(QObject *parent)
    : QSqlTableModel(parent)
{
  // qDebug() << "creating table";
  createTable();
  setTable(songsTableName);
  setEditStrategy(QSqlTableModel::OnManualSubmit);
  // make sure to call select else the model won't fill
  select();
}

QVariant SongSqlModel::data(const QModelIndex &index, int role) const {
  if (role < Qt::UserRole) {
    // qDebug() << role;
    return QSqlTableModel::data(index, role);
  }

  // qDebug() << role;
  const QSqlRecord sqlRecord = record(index.row());
  return sqlRecord.value(role - Qt::UserRole);
}

QHash<int, QByteArray> SongSqlModel::roleNames() const
{
    QHash<int, QByteArray> names;
    names[Qt::UserRole] = "id";
    names[Qt::UserRole + 1] = "title";
    names[Qt::UserRole + 2] = "lyrics";
    names[Qt::UserRole + 3] = "author";
    names[Qt::UserRole + 4] = "ccli";
    names[Qt::UserRole + 5] = "audio";
    names[Qt::UserRole + 6] = "vorder";
    names[Qt::UserRole + 7] = "background";
    names[Qt::UserRole + 8] = "backgroundType";
    names[Qt::UserRole + 9] = "horizontalTextAlignment";
    names[Qt::UserRole + 10] = "verticalTextAlignment";
    names[Qt::UserRole + 11] = "font";
    names[Qt::UserRole + 12] = "fontSize";
    return names;
}

void SongSqlModel::newSong() {
  qDebug() << "starting to add new song";
  int rows = rowCount();
  
  qDebug() << rows;
  QSqlRecord recorddata = record();
  recorddata.setValue("title", "new song");
  qDebug() << recorddata;

  if (insertRecord(rows, recorddata)) {
    submitAll();
  } else {
    qDebug() << lastError();
  };
}

void SongSqlModel::deleteSong(const int &row) {
  QSqlRecord recordData = record(row);
  if (recordData.isEmpty())
    return;

  removeRow(row);
  submitAll();
}

QVariantMap SongSqlModel::getSong(const int &row) {
  // this whole function returns all data in the song
  // regardless of it's length. When new things are added
  // it will still work without refactoring.
  const QModelIndex &parent = QModelIndex();
  QVariantMap data;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  const QModelIndex idx = this->index(row, 0, parent);
  qDebug() << "%%%%%%%%%";
  qDebug() << row;
  qDebug() << idx;
  qDebug() << "%%%%%%%%%";
  if( !idx.isValid() )
    return data;
  const QHash<int,QByteArray> rn = roleNames();
  // qDebug() << rn;
  QHashIterator<int,QByteArray> it(rn);
  while (it.hasNext()) {
    it.next();
    qDebug() << it.key() << ":" << it.value();
    data[it.value()] = idx.data(it.key());
  }
  return data;
}

QStringList SongSqlModel::getLyricList(const int &row) {
  QSqlQuery query("select id from songs");
  QList<int> ids;
  // qDebug() << row;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  qDebug() << "@@@@@@@@@@@@@@";
  qDebug() << id;
  qDebug() << "@@@@@@@@@@@@@@";
  QSqlRecord recordData = record(row);
  if (recordData.isEmpty()) {
    qDebug() << "this is not a song";
    QStringList empty;
    return empty;
  }

  QStringList rawLyrics = recordData.value("lyrics").toString().split("\n");
  qDebug() << "HERE ARE RAW LYRICS: " << rawLyrics;
  QStringList lyrics;

  QStringList vorder = recordData.value("vorder").toString().split(" ");
  // qDebug() << vorder;

  QStringList keywords = {"Verse 1", "Verse 2", "Verse 3", "Verse 4",
                          "Verse 5", "Verse 6", "Verse 7", "Verse 8",
                          "Chorus 1", "Chorus 2", "Chorus 3", "Chorus 4",
                          "Bridge 1", "Bridge 2", "Bridge 3", "Bridge 4",
                          "Intro 1", "Intro 2", "Ending 1", "Ending 2",
                          "Other 1", "Other 2", "Other 3", "Other 4"};

  bool firstItem = true;
  QString verse;
  QString vtitle;
  QString line;
  QMultiMap<QString, QString> verses;

  //TODO make sure to split empty line in verse into two slides

  // This first function pulls out each verse into our verses map
  // foreach (line, rawLyrics) {
  for (int i = 0; i < rawLyrics.length(); ++i) {
    // qDebug() << "##########################";
    // qDebug() << rawLyrics[i];
    // qDebug() << rawLyrics.length();
    // qDebug() << i;
    // qDebug() << "##########################";
    if (firstItem) {
      if (keywords.contains(rawLyrics[i])) {
        // qDebug() << "!!!!THIS IS FIRST LINE!!!!!";
        // qDebug() << rawLyrics[i];
        firstItem = false;
        vtitle = rawLyrics[i];
        continue;
      }
    } else if (keywords.contains(rawLyrics[i])) {
      // qDebug() << "!!!!THIS IS A VTITLE!!!!!";
      // qDebug() << verse;
      // qDebug() << rawLyrics[i];
      if (verse.contains("\n\n")) {
        verse = verse.trimmed();
        // qDebug() << "THIS IS A EMPTY SLIDE!" << verse;
        QStringList multiverses = verse.split("\n\n");
        foreach (verse, multiverses) {
          verses.insert(vtitle, verse);
          // qDebug() << verse;
        }
        verse.clear();
        multiverses.clear();
        vtitle = rawLyrics[i];
        continue;
      }
      verses.insert(vtitle, verse);
      verse.clear();
      vtitle = rawLyrics[i];
      continue;
    } else if (i + 1 == rawLyrics.length()) {
      // qDebug() << "!!!!LAST LINE!!!!!";

      verse.append(rawLyrics[i].trimmed() + "\n");
      if (verse.contains("\n\n")) {
        verse = verse.trimmed();
        // qDebug() << "THIS IS A EMPTY SLIDE!" << verse;
        QStringList multiverses = verse.split("\n\n");
        foreach (verse, multiverses) {
          verses.insert(vtitle, verse);
          // qDebug() << verse;
        }
        break;
      }

      verses.insert(vtitle, verse);
      // qDebug() << "&&&&&&&&&&&&";
      // qDebug() << "This is final line";
      // qDebug() << "and has been inserted";
      // qDebug() << verses.values(vtitle);
      // qDebug() << "&&&&&&&&&&&&";
      break;
    }
    // qDebug() << "THIS RAWLYRICS[I]";
    // qDebug() << rawLyrics[i];
    // qDebug() << "THIS VTITLE";
    // qDebug() << vtitle;
    verse.append(rawLyrics[i].trimmed() + "\n");
    // qDebug() << verse;
    // qDebug() << "APPENDED VERSE";
  }
  // qDebug() << verses;

  // let's check to see if there is a verse order, if not return the list given
  if (vorder.first().isEmpty()) {
    // qDebug() << "NO VORDER";
    foreach (verse, verses) {
      // qDebug() << verse;
      lyrics.append(verse);
    }
    // qDebug() << lyrics;
    return lyrics;
  }

  // this function appends the verse that matches the verse order from the map
  // first we run through every line and check to see if the line matches
  // an item in vorder, then we append lyrics from the verse that matches
  // the verse map we created earlier. It's a multi map so we need to append
  // them in reverse as they are added in last in first out order.
  foreach (const QString &vstr, vorder) {
    foreach (line, rawLyrics) {
      if (line.startsWith(vstr.at(0)) && line.endsWith(vstr.at(1))) {
        qDebug() << "**********************";
        qDebug() << vstr;
        qDebug() << line;
        qDebug() << "**********************";
        QList<QString> values = verses.values(line);
        for (int i = values.size(); i > 0;)
          lyrics.append(values.at(--i));
      }
    }
  }

  // qDebug() << lyrics;

  return lyrics;
}

int SongSqlModel::id() const {
  return m_id;
}

QString SongSqlModel::title() const {
  return m_title;
}

void SongSqlModel::setTitle(const QString &title) {
  if (title == m_title)
    return;
  
  m_title = title;

  select();
  emit titleChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateTitle(const int &row, const QString &title) {
  qDebug() << "Row is " << row;
  // QSqlQuery query("select id from songs");
  // QList<int> ids;
  // while (query.next()) {
  //   ids.append(query.value(0).toInt());
  //   // qDebug() << ids;
  // }
  // int id = ids.indexOf(row,0);

  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("title", title);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit titleChanged();
}

QString SongSqlModel::author() const {
  return m_author;
}

void SongSqlModel::setAuthor(const QString &author) {
  if (author == m_author)
    return;
  
  m_author = author;

  select();
  emit authorChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateAuthor(const int &row, const QString &author) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("author", author);
  setRecord(id, rowdata);
  submitAll();
  emit authorChanged();
}

QString SongSqlModel::lyrics() const {
  return m_lyrics;
}

void SongSqlModel::setLyrics(const QString &lyrics) {
  if (lyrics == m_lyrics)
    return;
  
  m_lyrics = lyrics;

  select();
  emit lyricsChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateLyrics(const int &row, const QString &lyrics) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << lyrics;
  rowdata.setValue("lyrics", lyrics);
  qDebug() << rowdata.value("lyrics");
  setRecord(id, rowdata);
  submitAll();
  emit lyricsChanged();
}

QString SongSqlModel::ccli() const {
  return m_ccli;
}

void SongSqlModel::setCcli(const QString &ccli) {
  if (ccli == m_ccli)
    return;
  
  m_ccli = ccli;

  select();
  emit ccliChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateCcli(const int &row, const QString &ccli) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("ccli", ccli);
  setRecord(id, rowdata);
  submitAll();
  emit ccliChanged();
}

QString SongSqlModel::audio() const {
  return m_audio;
}

void SongSqlModel::setAudio(const QString &audio) {
  if (audio == m_audio)
    return;
  
  m_audio = audio;

  select();
  emit audioChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateAudio(const int &row, const QString &audio) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("audio", audio);
  setRecord(id, rowdata);
  submitAll();
  emit audioChanged();
}

QString SongSqlModel::vorder() const { return m_vorder; }

void SongSqlModel::setVerseOrder(const QString &vorder) {
  if (vorder == m_vorder)
    return;

  m_vorder = vorder;

  select();
  emit vorderChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateVerseOrder(const int &row, const QString &vorder) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("vorder", vorder);
  setRecord(id, rowdata);
  submitAll();
  emit vorderChanged();
}

QString SongSqlModel::background() const { return m_background; }

void SongSqlModel::setBackground(const QString &background) {
  if (background == m_background)
    return;

  m_background = background;

  select();
  emit backgroundChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateBackground(const int &row, const QString &background) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("background", background);
  setRecord(id, rowdata);
  submitAll();
  emit backgroundChanged();
}

QString SongSqlModel::backgroundType() const { return m_backgroundType; }

void SongSqlModel::setBackgroundType(const QString &backgroundType) {
  if (backgroundType == m_backgroundType)
    return;

  m_backgroundType = backgroundType;

  select();
  emit backgroundTypeChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateBackgroundType(const int &row, const QString &backgroundType) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  rowdata.setValue("backgroundType", backgroundType);
  setRecord(id, rowdata);
  submitAll();
  emit backgroundTypeChanged();
}

QString SongSqlModel::horizontalTextAlignment() const {
  return m_horizontalTextAlignment;
}

void SongSqlModel::setHorizontalTextAlignment(const QString &horizontalTextAlignment) {
  if (horizontalTextAlignment == m_horizontalTextAlignment)
    return;
  
  m_horizontalTextAlignment = horizontalTextAlignment;

  select();
  emit horizontalTextAlignmentChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateHorizontalTextAlignment(const int &row, const QString &horizontalTextAlignment) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("horizontalTextAlignment", horizontalTextAlignment);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit horizontalTextAlignmentChanged();
}

QString SongSqlModel::verticalTextAlignment() const {
  return m_verticalTextAlignment;
}

void SongSqlModel::setVerticalTextAlignment(const QString &verticalTextAlignment) {
  if (verticalTextAlignment == m_verticalTextAlignment)
    return;
  
  m_verticalTextAlignment = verticalTextAlignment;

  select();
  emit verticalTextAlignmentChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateVerticalTextAlignment(const int &row, const QString &verticalTextAlignment) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("verticalTextAlignment", verticalTextAlignment);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit verticalTextAlignmentChanged();
}

QString SongSqlModel::font() const {
  return m_font;
}

void SongSqlModel::setFont(const QString &font) {
  if (font == m_font)
    return;
  
  m_font = font;

  select();
  emit fontChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateFont(const int &row, const QString &font) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("font", font);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit fontChanged();
}

int SongSqlModel::fontSize() const {
  return m_fontSize;
}

void SongSqlModel::setFontSize(const int &fontSize) {
  if (fontSize == m_fontSize)
    return;
  
  m_fontSize = fontSize;

  select();
  emit fontSizeChanged();
}

// This function is for updating the lyrics from outside the delegate
void SongSqlModel::updateFontSize(const int &row, const int &fontSize) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from songs");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("fontSize", fontSize);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit fontSizeChanged();
}

QModelIndex SongSqlModel::idx(int row) {
  QModelIndex idx = index(row, 0);
  // qDebug() << idx;
  return idx;
}

// SongProxyModel

SongProxyModel::SongProxyModel(QObject *parent)
  :QSortFilterProxyModel(parent)
{
  m_songModel = new SongModel;
  m_songModel->setup();
  // m_selectionModel = new QItemSelectionModel;
  // m_selectionModel->setModel(this);
  setSourceModel(m_songModel);
  setDynamicSortFilter(true);
  setFilterRole(Qt::UserRole + 1);
  setFilterCaseSensitivity(Qt::CaseInsensitive);
}

SongModel *SongProxyModel::songModel() {
  return m_songModel;
}

QModelIndex SongProxyModel::idx(int row) {
  QModelIndex idx = index(row, 0);
  // qDebug() << idx;
  return idx;
}

QModelIndex SongProxyModel::modelIndex(int row) {
  QModelIndex idx = m_songModel->index(mapToSource(index(row, 0)).row());
  return idx;
}

QStringList SongProxyModel::getLyricList(const int &row) {
  QStringList lyrics = m_songModel->getLyricList(mapToSource(index(row, 0)).row());
  return lyrics;
}

QVariantMap SongProxyModel::getSong(const int &row) {
  QVariantMap song = m_songModel->getItem(mapToSource(index(row, 0)).row());
  qDebug() << song;
  return song;
}

void SongProxyModel::newSong() {
  m_songModel->newSong();
}

void SongProxyModel::deleteSong(const int &row) {
  auto model = qobject_cast<SongModel *>(sourceModel());
  model->removeItem(row);
}

void SongProxyModel::deleteSongs(const QVector<int> &rows) {
  auto model = qobject_cast<SongModel *>(sourceModel());
  qDebug() << "DELETING!!!!!!!!!!!!!!!!!!!!!!!" << rows;
  for (int i = rows.size() - 1; i >= 0; i--) {
    qDebug() << "deleting" << rows.at(i);
    model->removeItem(rows.at(i));
  }
}

