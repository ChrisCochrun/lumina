#ifndef SONGSQLMODEL_H
#define SONGSQLMODEL_H

#include <QSqlTableModel>
#include <qabstractitemmodel.h>
#include <qobjectdefs.h>
#include <qqml.h>
#include <qvariant.h>

class SongSqlModel : public QSqlTableModel
{
  Q_OBJECT
  Q_PROPERTY(int id READ id)
  Q_PROPERTY(QString title READ title WRITE setTitle NOTIFY titleChanged)
  Q_PROPERTY(QString lyrics READ lyrics WRITE setLyrics NOTIFY lyricsChanged)
  Q_PROPERTY(QString author READ author WRITE setAuthor NOTIFY authorChanged)
  Q_PROPERTY(QString ccli READ ccli WRITE setCcli NOTIFY ccliChanged)
  Q_PROPERTY(QString audio READ audio WRITE setAudio NOTIFY audioChanged)
  Q_PROPERTY(QString vorder READ vorder WRITE setVerseOrder NOTIFY vorderChanged)
  QML_ELEMENT

public:
  SongSqlModel(QObject *parent = 0);

  int id() const;
  QString title() const;
  QString lyrics() const;
  QString author() const;
  QString ccli() const;
  QString audio() const;
  QString vorder() const;

  void setTitle(const QString &title);
  void setLyrics(const QString &lyrics);
  void setAuthor(const QString &author);
  void setCcli(const QString &ccli);
  void setAudio(const QString &audio);
  void setVerseOrder(const QString &vorder);

  Q_INVOKABLE void updateTitle(const int &row, const QString &title);
  Q_INVOKABLE void updateLyrics(const int &row, const QString &lyrics);
  Q_INVOKABLE void updateAuthor(const int &row, const QString &author);
  Q_INVOKABLE void updateCcli(const int &row, const QString &ccli);
  Q_INVOKABLE void updateAudio(const int &row, const QString &audio);
  Q_INVOKABLE void updateVerseOrder(const int &row, const QString &vorder);

  Q_INVOKABLE void newSong();

  QVariant data(const QModelIndex &index, int role) const override;
  QHash<int, QByteArray> roleNames() const override;

signals:
    void titleChanged();
    void lyricsChanged();
    void authorChanged();
    void ccliChanged();
    void audioChanged();
    void vorderChanged();

private:
    int m_id;
    QString m_title;
    QString m_lyrics;
    QString m_author;
    QString m_ccli;
    QString m_audio;
    QString m_vorder;
};

#endif //SONGSQLMODEL_H
