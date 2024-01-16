#include "presentationsqlmodel.h"

#include <QDateTime>
#include <QDebug>
#include <QSqlError>
#include <QSqlRecord>
#include <QSqlQuery>
#include <QSql>
// #include <QPdfDocument>
#include <QSqlDatabase>
#include <QFileInfo>
#include <qabstractitemmodel.h>
#include <qdebug.h>
#include <qnamespace.h>
#include <qobject.h>
#include <qobjectdefs.h>
#include <qsqlrecord.h>
#include <qurl.h>
#include <qvariant.h>

static const char *presentationsTableName = "presentations";

static void createPresentationTable()
{
  QSqlQuery query;
  if(QSqlDatabase::database().tables().contains(presentationsTableName)) {
    // query.exec("DROP TABLE 'presentations'");
    // qDebug() << query.lastQuery();
    return;
  }

  if (!query.exec("CREATE TABLE IF NOT EXISTS 'presentations' ("
                  "  'id' INTEGER NOT NULL,"
                  "  'title' TEXT NOT NULL,"
                  "  'filePath' TEXT NOT NULL,"
                  "  'pageCount' INTEGER,"
                  "  PRIMARY KEY(id))")) {
    qFatal("Failed to query database: %s",
           qPrintable(query.lastError().text()));
  }
  qDebug() << query.lastQuery();
  qDebug() << "inserting into presentations";

  query.exec("INSERT INTO presentations (title, filePath) VALUES ('Dec 180', 'file:///home/chris/nextcloud/tfc/openlp/5 slides-1.pdf')");
  qDebug() << query.lastQuery();
  query.exec("INSERT INTO presentations (title, filePath) VALUES ('No TFC', "
             "'file:///home/chris/nextcloud/tfc/openlp/5 slides-2.pdf')");

  query.exec("select * from presentations");
  qDebug() << query.lastQuery();
}

PresentationSqlModel::PresentationSqlModel(QObject *parent) : QSqlTableModel(parent) {
  qDebug() << "creating presentation table";
  createPresentationTable();
  setTable(presentationsTableName);
  setEditStrategy(QSqlTableModel::OnManualSubmit);
  // make sure to call select else the model won't fill
  select();
}

QVariant PresentationSqlModel::data(const QModelIndex &index, int role) const {
  if (role < Qt::UserRole) {
    return QSqlTableModel::data(index, role);
  }

  // qDebug() << role;
  const QSqlRecord sqlRecord = record(index.row());
  return sqlRecord.value(role - Qt::UserRole);
}

QHash<int, QByteArray> PresentationSqlModel::roleNames() const
{
    QHash<int, QByteArray> names;
    names[Qt::UserRole] = "id";
    names[Qt::UserRole + 1] = "title";
    names[Qt::UserRole + 2] = "filePath";
    names[Qt::UserRole + 3] = "pageCount";
    return names;
}

void PresentationSqlModel::newPresentation(const QUrl &filePath, int pageCount) {
  qDebug() << "adding new presentation";
  int rows = rowCount();

  qDebug() << rows;
  QSqlRecord recordData = record();
  QFileInfo fileInfo = filePath.toString();
  QString title = fileInfo.baseName();
  recordData.setValue("title", title);
  recordData.setValue("filePath", filePath);
  recordData.setValue("pageCount", pageCount);

  if (insertRecord(rows, recordData)) {
    submitAll();
  } else {
    qDebug() << lastError();
  };
}

void PresentationSqlModel::deletePresentation(const int &row) {
  QSqlRecord recordData = record(row);
  if (recordData.isEmpty())
    return;

  removeRow(row);
  submitAll();
}

int PresentationSqlModel::id() const {
  return m_id;
}

QString PresentationSqlModel::title() const {
  return m_title;
}

void PresentationSqlModel::setTitle(const QString &title) {
  if (title == m_title)
    return;
  
  m_title = title;

  select();
  emit titleChanged();
}

// This function is for updating the title from outside the delegate
void PresentationSqlModel::updateTitle(const int &row, const QString &title) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from presentations");
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

QUrl PresentationSqlModel::filePath() const {
  return m_filePath;
}

void PresentationSqlModel::setFilePath(const QUrl &filePath) {
  if (filePath == m_filePath)
    return;
  
  m_filePath = filePath;

  select();
  emit filePathChanged();
}

// This function is for updating the filepath from outside the delegate
void PresentationSqlModel::updateFilePath(const int &row, const QUrl &filePath) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from presentations");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("filePath", filePath);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit filePathChanged();
}

int PresentationSqlModel::pageCount() const {
  return m_pageCount;
}

void PresentationSqlModel::setPageCount(const int &pageCount) {
  if (pageCount == m_pageCount)
    return;
  
  m_pageCount = pageCount;

  select();
  emit pageCountChanged();
}

// This function is for updating the pageCount from outside the delegate
void PresentationSqlModel::updatePageCount(const int &row, const int &pageCount) {
  qDebug() << "Row is " << row;
  QSqlQuery query("select id from presentations");
  QList<int> ids;
  while (query.next()) {
    ids.append(query.value(0).toInt());
    // qDebug() << ids;
  }
  int id = ids.indexOf(row,0);

  QSqlRecord rowdata = record(id);
  qDebug() << rowdata;
  rowdata.setValue("pageCount", pageCount);
  setRecord(id, rowdata);
  qDebug() << rowdata;
  submitAll();
  emit pageCountChanged();
}

QVariantMap PresentationSqlModel::getPresentation(const int &row) {
  // qDebug() << "Row we are getting is " << row;
  // QUrl presentation;
  // QSqlRecord rec = record(row);
  // qDebug() << rec.value("filePath").toUrl();
  // // presentation.append(rec.value("title"));
  // // presentation.append(rec.value("filePath"));
  // presentation = rec.value("filePath").toUrl();
  // return presentation;

  QVariantMap data;
  const QModelIndex idx = this->index(row,0);
  // qDebug() << idx;
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
// PresentationProxyModel

PresentationProxyModel::PresentationProxyModel(QObject *parent)
  :QSortFilterProxyModel(parent)
{
  m_presentationModel = new PresentationModel;
  m_presentationModel->setup();
  setSourceModel(m_presentationModel);
  setDynamicSortFilter(true);
  setFilterRole(Qt::UserRole + 1);
  setFilterCaseSensitivity(Qt::CaseInsensitive);
}

PresentationModel *PresentationProxyModel::presentationModel() {
  return m_presentationModel;
}

QModelIndex PresentationProxyModel::idx(int row) {
  QModelIndex idx = index(row, 0);
  // qDebug() << idx;
  return idx;
}

QVariantMap PresentationProxyModel::getPresentation(const int &row) {
  qDebug() << "Getting Presentation through cpp";
  auto model = qobject_cast<PresentationModel *>(sourceModel());
  QVariantMap presentation = model->getItem(mapToSource(index(row, 0)).row());
  return presentation;
}

void PresentationProxyModel::deletePresentation(const int &row) {
  auto model = qobject_cast<PresentationModel *>(sourceModel());
  model->removeItem(row);
}

void PresentationProxyModel::deletePresentations(const QVector<int> &rows) {
  auto model = qobject_cast<PresentationModel *>(sourceModel());
  qDebug() << "DELETING!!!!!!!!!!!!!!!!!!!!!!!" << rows;
  for (int i = rows.size() - 1; i >= 0; i--) {
    qDebug() << "deleting" << rows.at(i);
    model->removeItem(rows.at(i));
  }
}
