#include "serviceitemmodel.h"
#include "serviceitem.h"
#include "filemanager.h"
#include "slidemodel.h"
#include <qabstractitemmodel.h>
#include <qglobal.h>
#include <qnamespace.h>
#include <qvariant.h>
#include <ktar.h>
#include <KCompressionDevice>
#include <KArchiveDirectory>
#include <KArchiveFile>
#include <KArchiveEntry>
#include <QDebug>
#include <QJsonArray>
#include <QJsonObject>
#include <QJsonDocument>
#include <QFile>
#include <QMap>
#include <QTemporaryFile>
#include <QDir>
#include <QUrl>
#include <QSettings>
#include <QStandardPaths>
#include <QImage>

#include "cxx-qt-gen/slide_model.cxxqt.h"
#include "cxx-qt-gen/service_item_model.cxxqt.h"

ServiceItemModel::ServiceItemModel(QObject *parent)
    : QAbstractListModel(parent) {
}

int ServiceItemModel::rowCount(const QModelIndex &parent) const {
  // For list models only the root node (an invalid parent) should return the
  // list's size. For all other (valid) parents, rowCount() should return 0 so
  // that it does not become a tree model.
  if (parent.isValid())
    return 0;

  // FIXME: Implement me!
  return m_items.size();
}

QVariant ServiceItemModel::data(const QModelIndex &index, int role) const {
  if (!index.isValid())
    return QVariant();

  ServiceItem *item = m_items[index.row()];
  switch (role) {
  case NameRole:
    return item->name();
  case TypeRole:
    return item->type();
  case BackgroundRole:
    return item->background();
  case BackgroundTypeRole:
    return item->backgroundType();
  case TextRole:
    return item->text();
  case AudioRole:
    return item->audio();
  case FontRole:
    return item->font();
  case FontSizeRole:
    return item->fontSize();
  case SlideNumberRole:
    return item->slideNumber();
  case ActiveRole:
    return item->active();
  case SelectedRole:
    return item->selected();
  case LoopRole:
    return item->loop();
  default:
    return QVariant();
  }
}

QHash<int, QByteArray> ServiceItemModel::roleNames() const {
  static QHash<int, QByteArray> mapping{{NameRole, "name"},
                                        {TypeRole, "type"},
                                        {BackgroundRole, "background"},
                                        {BackgroundTypeRole, "backgroundType"},
                                        {TextRole, "text"},
                                        {AudioRole, "audio"},
                                        {FontRole, "font"},
                                        {FontSizeRole, "fontSize"},
                                        {SlideNumberRole, "slideNumber"},
                                        {ActiveRole, "active"},
                                        {SelectedRole, "selected"},
                                        {LoopRole, "loop"}};

  return mapping;
}

bool ServiceItemModel::setData(const QModelIndex &index, const QVariant &value,
                               int role) {

  ServiceItem *item = m_items[index.row()];
  bool somethingChanged = false;

  switch (role) {
  case NameRole:
    if (item->name() != value.toString()) {
      item->setName(value.toString());
      somethingChanged = true;
    }
    break;
  case TypeRole:
    if (item->type() != value.toString()) {
      item->setType(value.toString());
      somethingChanged = true;
    }
    break;
  case BackgroundRole:
    if (item->background() != value.toString()) {
      item->setBackground(value.toString());
      somethingChanged = true;
    }
    break;
  case BackgroundTypeRole:
    if (item->backgroundType() != value.toString()) {
      item->setBackgroundType(value.toString());
      somethingChanged = true;
    }
    break;
  case TextRole:
    if (item->text() != value.toStringList()) {
      item->setText(value.toStringList());
      somethingChanged = true;
    }
    break;
  case AudioRole:
    if (item->audio() != value.toString()) {
      item->setAudio(value.toString());
      somethingChanged = true;
    }
    break;
  case FontRole:
    if (item->font() != value.toString()) {
      item->setFont(value.toString());
      somethingChanged = true;
    }
    break;
  case FontSizeRole:
    if (item->fontSize() != value.toInt()) {
      item->setFontSize(value.toInt());
      somethingChanged = true;
    }
    break;
  case SlideNumberRole:
    if (item->slideNumber() != value.toInt()) {
      item->setSlideNumber(value.toInt());
      somethingChanged = true;
    }
    break;
  case ActiveRole:
    if (item->active() != value.toBool()) {
      item->setActive(value.toBool());
      somethingChanged = true;
    }
    break;
  case SelectedRole:
    if (item->selected() != value.toBool()) {
      item->setSelected(value.toBool());
      somethingChanged = true;
    }
    break;
  case LoopRole:
    if (item->loop() != value.toBool()) {
      item->setLoop(value.toBool());
      somethingChanged = true;
    }
    break;
    if (somethingChanged) {
      emit dataChanged(index, index, QVector<int>() << role);
      return true;
    }
  }

  return false;
}

Qt::ItemFlags ServiceItemModel::flags(const QModelIndex &index) const {
  if (!index.isValid())
    return Qt::NoItemFlags;

  return Qt::ItemIsEditable; // FIXME: Implement me!
}

void ServiceItemModel::addItem(ServiceItem *item) {
  const int index = m_items.size();
  qDebug() << index;
  // foreach (item, m_items) {
  //   qDebug() << item;
  // }
  beginInsertRows(QModelIndex(), index, index);
  m_items.append(item);
  endInsertRows();
}

void ServiceItemModel::insertItem(const int &index, ServiceItem *item) {
  beginInsertRows(this->index(index).parent(), index, index);
  m_items.insert(index, item);
  endInsertRows();
  qDebug() << "Success";
}

void ServiceItemModel::addItem(const QString &name, const QString &type,
                               const QString &background, const QString &backgroundType,
                               const QStringList &text, const QString &audio,
                               const QString &font, const int &fontSize,
                               const int &slideNumber, const bool &loop) {
  qDebug() << "*************************";
  qDebug() << "Plain adding item: " << name;
  qDebug() << "*************************";
  ServiceItem *item = new ServiceItem(name, type, background, backgroundType,
                                      text, audio, font, fontSize, slideNumber, loop);
  item->setSelected(false);
  item->setActive(false);
  addItem(item);

  QVariantMap itm;
  const QModelIndex idx = this->index(rowCount() - 1);
  qDebug() << idx;
  qDebug() << rowCount();
  if( idx.isValid() ) {
    const QHash<int,QByteArray> rn = roleNames();
    // qDebug() << rn;
    QHashIterator<int,QByteArray> it(rn);
    while (it.hasNext()) {
      it.next();
      qDebug() << "trains";
      qDebug() << it.key() << ":" << it.value() << ":" << idx.data(it.key());
      itm[it.value()] = idx.data(it.key());
    }
  } else
    qDebug() << "idx isn't valid";
  qDebug() << "*&";
  qDebug() << itm;
  qDebug() << "*&";
  // emit itemAdded(rowCount() - 1, *item);
  emit itemAddedRust(rowCount() - 1, itm);
  qDebug() << "EMITTED ITEM ADDED" << rowCount();
  qDebug() << "#################################";
  qDebug() << name << type << font << fontSize << slideNumber;
  qDebug() << "#################################";
}

void ServiceItemModel::insertItem(const int &index, const QString &name,
                                  const QString &type,const QString &background,
                                  const QString &backgroundType,const QStringList &text,
                                  const QString &audio, const QString &font,
                                  const int &fontSize, const int &slideNumber,
                                  const bool &loop) {
  qDebug() << "*************************";
  qDebug() << "Inserting serviceItem: " << name << " and index is " << index;
  qDebug() << "*************************";
  ServiceItem *item = new ServiceItem(name, type, background, backgroundType,
                                      text, audio, font, fontSize, slideNumber, loop);
  item->setSelected(false);
  item->setActive(false);
  insertItem(index, item);

  QVariantMap itm;
  const QModelIndex idx = this->index(index);
  qDebug() << idx;
  if( idx.isValid() ) {
    const QHash<int,QByteArray> rn = roleNames();
    // qDebug() << rn;
    QHashIterator<int,QByteArray> it(rn);
    while (it.hasNext()) {
      it.next();
      qDebug() << "trains";
      qDebug() << it.key() << ":" << it.value() << ":" << idx.data(it.key());
      itm[it.value()] = idx.data(it.key());
    }
  } else
    qDebug() << "idx isn't valid";

  // emit itemInserted(index, *item);
  emit itemInsertedRust(index, itm);

  qDebug() << "EMITTED ITEM INSERTED";
  qDebug() << "#################################";
  qDebug() << "INSERTING SERVICE ITEM!";
  qDebug() << name << type << font << fontSize << slideNumber << index;
  qDebug() << "#################################";
}

void ServiceItemModel::removeItem(int index) {
  beginRemoveRows(QModelIndex(), index, index);
  m_items.removeAt(index);
  endRemoveRows();
}

void ServiceItemModel::removeItems() {
  for (int i = m_items.length() - 1; i > -1; i--) {
    QModelIndex idx = index(i);
    ServiceItem *item = m_items[idx.row()];
    if (item->selected()) {
      qDebug() << "Removing item:" << i;
      beginRemoveRows(QModelIndex(), i, i);
      m_items.removeAt(i);
      endRemoveRows();
      QVariantMap map = getItem(i);
      emit rowRemoved(i, *item);
      emit rowRemovedRust(i, map);
      qDebug() << "emitted removal of item:" << item->name();
    }
  }

}

bool ServiceItemModel::moveRows(int sourceIndex, int destIndex, int count) {
  qDebug() << sourceIndex;
  qDebug() << destIndex;

  const int lastIndex = rowCount() - 1;

  if (sourceIndex == destIndex
      || (sourceIndex < 0 || sourceIndex > lastIndex)
      || (destIndex < 0 || destIndex > lastIndex)) {
    return false;
  }

  const QModelIndex parent = index(sourceIndex).parent();
  const bool isMoveDown = destIndex > sourceIndex;

  if (!beginMoveRows(parent, sourceIndex, sourceIndex + count - 1,
                     parent, isMoveDown ? destIndex + 1 : destIndex)) {
    qDebug() << "Can't move rows";
    return false;
  }
    
  qDebug() << "starting move: " << "source: " << sourceIndex << "dest: " << destIndex;

  m_items.move(sourceIndex, destIndex);

  endMoveRows();

  QModelIndex idx = index(destIndex);
  ServiceItem *item = m_items[idx.row()];
  emit rowMoved(sourceIndex, destIndex, *item);
  QVariantMap map = getItem(destIndex);
  emit rowMovedRust(sourceIndex, destIndex, map);

  return true;
}

bool ServiceItemModel::moveDown(int id) {
  qDebug() << index(id).row();
  qDebug() << index(id + 1).row();
  QModelIndex parent = index(id).parent();

  bool begsuc = beginMoveRows(parent, id,
                              id, parent, id + 2);
  if (begsuc) {
    int dest = id + 1;
    if (dest >= m_items.size())
      {
        qDebug() << "dest too big, moving to end";
        m_items.move(id, m_items.size() - 1);
      }
    else
      m_items.move(id, dest);
    endMoveRows();
    return true;
  }
  return false;
}

bool ServiceItemModel::moveUp(int id) {
  qDebug() << index(id).row();
  qDebug() << index(id - 1).row();
  QModelIndex parent = index(id).parent();

  bool begsuc = beginMoveRows(parent, id,
                              id, parent, id - 1);
  if (begsuc) {
    int dest = id - 1;
    if (dest <= -1)
      {
        qDebug() << "dest too big, moving to beginning";
        m_items.move(id, 0);
      }
    else
      m_items.move(id, dest);
    endMoveRows();
    return true;
  }

  return false;
}

QVariantMap ServiceItemModel::getRust(int index, ServiceItemMod *rustModel) const {
  QVariantMap item = rustModel->getItem(index);
  return item;
}

QVariantMap ServiceItemModel::getItem(int index) const {
  QVariantMap data;
  const QModelIndex idx = this->index(index,0);
  // qDebug() << idx;
  if( !idx.isValid() )
    return data;
  const QHash<int,QByteArray> rn = roleNames();
  // qDebug() << rn;
  QHashIterator<int,QByteArray> it(rn);
  while (it.hasNext()) {
    it.next();
    // qDebug() << it.key() << ":" << it.value();
    data[it.value()] = idx.data(it.key());
  }
  return data;
}

QVariantList ServiceItemModel::getItems() {
  QVariantList data;
  ServiceItem * item;
  foreach (item, m_items) {
    qDebug() << item->name();
    QVariantMap itm;
    itm["name"] = item->name();
    itm["type"] = item->type();
    itm["background"] = item->background();
    itm["backgroundType"] = item->backgroundType();
    itm["text"] = item->text();
    itm["audio"] = item->audio();
    itm["font"] = item->font();
    itm["fontSize"] = item->fontSize();
    itm["slideNumber"] = item->slideNumber();
    itm["selected"] = item->selected();
    itm["loop"] = item->loop();
    itm["active"] = item->active();
    data.append(itm);
  }
  qDebug() << "$$$$$$$$$$$$$$$$$$$$$$$$$$$";
  qDebug() << data;
  qDebug() << "$$$$$$$$$$$$$$$$$$$$$$$$$$$";
  return data;
}

bool ServiceItemModel::select(int id) {
  for (int i = 0; i < m_items.length(); i++) {
    QModelIndex idx = index(i);
    ServiceItem *item = m_items[idx.row()];
    if (item->selected()) {
      item->setSelected(false);
      qDebug() << "################";
      qDebug() << "deselected" << item->name();
      qDebug() << "################";
      emit dataChanged(idx, idx, QVector<int>() << SelectedRole);
    }
  }
  QModelIndex idx = index(id);
  ServiceItem *item = m_items[idx.row()];
  item->setSelected(true);
  qDebug() << "################";
  qDebug() << "selected" << item->name();
  qDebug() << "################";
  emit dataChanged(idx, idx, QVector<int>() << SelectedRole);
  return true;
}

bool ServiceItemModel::selectItems(QVariantList items) {
  qDebug() << "Let's select some items!";
  for (int i = 0; i < m_items.length(); i++) {
    QModelIndex idx = index(i);
    ServiceItem *item = m_items[idx.row()];
    if (item->selected()) {
      item->setSelected(false);
      qDebug() << "################";
      qDebug() << "deselected" << item->name();
      qDebug() << "################";
      emit dataChanged(idx, idx, QVector<int>() << SelectedRole);
    }
  }
  qDebug() << "All things have been deselected";
  foreach (QVariant it, items) {
    int i = it.toInt();
    QModelIndex idx = index(i);
    ServiceItem *item = m_items[idx.row()];
    if (!item->selected()) {
      item->setSelected(true);
      qDebug() << "################";
      qDebug() << "selected" << item->name();
      qDebug() << "################";
      emit dataChanged(idx, idx, QVector<int>() << SelectedRole);
    }
  }
  return true;
}

bool ServiceItemModel::activate(int id) {
  QModelIndex idx = index(id);
  ServiceItem *item = m_items[idx.row()];

  for (int i = 0; i < m_items.length(); i++) {
    QModelIndex idx = index(i);
    ServiceItem *itm = m_items[idx.row()];
    if (itm->active()) {
      itm->setActive(false);
      qDebug() << "################";
      qDebug() << "deactivated" << itm->name();
      qDebug() << "################";
      emit dataChanged(idx, idx, QVector<int>() << ActiveRole);
    }
  }

  item->setActive(true);
  qDebug() << "################";
  qDebug() << "activated" << item->name();
  qDebug() << "################";
  emit dataChanged(idx, idx, QVector<int>() << ActiveRole);
  return true;
}

bool ServiceItemModel::deactivate(int id) {
  QModelIndex idx = index(id);
  ServiceItem *item = m_items[idx.row()];

  item->setActive(false);
  qDebug() << "################";
  qDebug() << "deactivated" << item->name();
  qDebug() << "################";
  emit dataChanged(idx, idx, QVector<int>() << ActiveRole);
  return true;
}

bool ServiceItemModel::save(QUrl file) {
  qDebug() << "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@";
  qDebug() << "Saving...";
  qDebug() << "File path is: " << file.toString();
  qDebug() << "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@";

  //first we'll get a json representation of our serviceList
  //save that to a temp file in case we need it but also convert
  //it to a byte array just before putting it into the archive

  QJsonArray jsonData;

  // save all the data and files in jsonData as just the base name
  // so that they are properly mapped in the resulting archive
  for (int i = 0; i < m_items.length(); i++) {
    // qDebug() << serviceList[i];
    QMap<QString, QVariant> item;
    qDebug() << m_items[i]->name();

    item.insert("name", m_items[i]->name());
    item.insert("background", m_items[i]->background());
    item.insert("backgroundType", m_items[i]->backgroundType());
    item.insert("audio", m_items[i]->audio());
    item.insert("font", m_items[i]->font());
    item.insert("fontSize", m_items[i]->fontSize());
    item.insert("slideNumber", m_items[i]->slideNumber());
    item.insert("text", m_items[i]->text());
    item.insert("type", m_items[i]->type());
    item.insert("loop", m_items[i]->loop());

    qDebug() << "AUDIO IS: " << item.value("audio").toString();
    QFileInfo audioFile = item.value("audio").toString();
    qDebug() << audioFile.fileName();
    item["flatAudio"] = audioFile.fileName();
    qDebug() << "AUDIO IS NOW: " << item.value("audio").toString();

    QFileInfo backgroundFile = item.value("background").toString();
    item["flatBackground"] = backgroundFile.fileName();
    qDebug() << "BACKGRUOND IS: " << item.value("background").toString();
    // qDebug() << serviceList[i].value();
    QJsonObject obj = QJsonObject::fromVariantMap(item);
    qDebug() << obj;
    jsonData.insert(i, obj);
  }

  qDebug() << jsonData;
  QJsonDocument jsonText(jsonData);
  QTemporaryFile jsonFile;

  if (!jsonFile.exists())
    qDebug() << "NOT EXISTS!";

  if (!jsonFile.open())
    return false;

  //finalize the temp json file, in case something goes wrong in the
  //archive, we'll have this to jump back to
  jsonFile.write(jsonText.toJson());
  qDebug() << jsonFile.fileName();
  jsonFile.close();

  //now we create our archive file and set it's parameters
  QString filename = file.toString().right(file.toString().size() - 7);
  qDebug() << filename;

  QString tarname;
  if (filename.endsWith(".pres")) {
    qDebug() << "Perfect just go with it!";
    tarname = filename;
  } else
    tarname = filename + ".pres";

  KTar tar(tarname, "application/zstd");

  if (tar.open(QIODevice::WriteOnly)) {
    qDebug() << tar.isOpen();

    //write our json data to the archive
    tar.writeFile("servicelist.json",
                  jsonText.toJson());

    //let's add the backgrounds and audios to the archive
    for (int i = 0; i < m_items.size(); i++) {
      qDebug() << m_items[i]->name();
      QString background = m_items[i]->background();
      QString backgroundFile = background.right(background.size() - 5);
      qDebug() << backgroundFile;
      QString audio = m_items[i]->audio();
      QString audioFile = audio.right(audio.size() - 5);
      qDebug() << audioFile;

      //here we need to cut off all the directories before
      //adding into the archive
      tar.addLocalFile(backgroundFile,
                       backgroundFile.right(backgroundFile.size() -
                                            backgroundFile.lastIndexOf("/") - 1));
      tar.addLocalFile(audioFile,
                       audioFile.right(audioFile.size() -
                                       audioFile.lastIndexOf("/") - 1));
    }

    //close the archive so that everything is done
    tar.close();

    QSettings settings;
    settings.setValue("lastSaveFile", file);

    settings.sync();

    qDebug() << settings.value("lastSaveFile");
    return true;
  }

  
  return false;
}

bool ServiceItemModel::load(QUrl file) {
  qDebug() << "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@";
  qDebug() << "Loading...";
  qDebug() << "File path is: " << file.toString();
  qDebug() << "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@";

  QFileInfo loadInfo = QFileInfo(file.toLocalFile());
  if (!loadInfo.exists())
    return false;

  QString fileUrl = file.toString().right(file.toString().size() - 7);
  KTar tar(fileUrl);

  if (tar.open(QIODevice::ReadOnly)){
    qDebug() << tar.isOpen();
    const KArchiveDirectory *dir = tar.directory();

    const KArchiveEntry *e = dir->entry("servicelist.json");
    if (!e) {
      qDebug() << "File not found!";
    }
    const KArchiveFile *f = static_cast<const KArchiveFile *>(e);
    QByteArray arr(f->data());
    QJsonDocument jsonText = QJsonDocument::fromJson(arr);
    qDebug() << jsonText; // the file contents

    QJsonArray array = jsonText.array();

    QVariantList serviceList = array.toVariantList();
    qDebug() << serviceList;

    // now lets remove all items from current list and add loaded ones
    clearAll();

    for (int i = 0; i < serviceList.length(); i++) {
      // int id = serviceList
      qDebug() << "*********************************";
      qDebug() << serviceList[i].toMap();
      qDebug() << "*********************************";

      QMap item = serviceList[i].toMap();

      QString backgroundString = item.value("background").toString();
      QFileInfo backgroundFile = backgroundString.right(backgroundString.size() - 7);

      QString audioString = item.value("audio").toString();
      QFileInfo audioFile = audioString.right(audioString.size() - 7);

      qDebug() << "POOPPOPOPOPOPOPOPOPOPOPOPOPO";
      qDebug() << backgroundFile;
      qDebug() << backgroundFile.exists();
      qDebug() << audioFile;
      qDebug() << audioFile.exists();
      qDebug() << "POOPPOPOPOPOPOPOPOPOPOPOPOPO";

      QString realBackground;
      QString realAudio;

      QFileInfo serviceFile = file.toString().right(file.toString().size() - 7);
      QString serviceName = serviceFile.baseName();
      QDir localDir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
      localDir.mkdir(serviceName);
      QDir serviceDir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation)
        + "/" + serviceName;
      qDebug() << serviceDir.path();

      realBackground = backgroundString;
      realAudio = audioString;
      // If the background file is on disk use that, else use the one in archive
      if (!backgroundFile.exists() && backgroundString.length() > 0) {
        const KArchiveEntry *e = dir->entry(backgroundFile.fileName());
        if (!e) {
          qDebug() << "Background File not found!";
          continue;
        }
        const KArchiveFile *f = static_cast<const KArchiveFile *>(e);
        if (!f->copyTo(serviceDir.path()))
          qDebug() << "FILE COULDN'T BE CREATED!";

        QFileInfo bgFile = serviceDir.path() + "/" + backgroundFile.fileName();

        qDebug() << bgFile.filePath();

        realBackground = bgFile.filePath();
      }

      // If the audio file is on disk use that, else use the one in archive
      if (!audioFile.exists() && audioString.length() > 0) {
        const KArchiveEntry *e = dir->entry(audioFile.fileName());
        if (!e) {
          qDebug() << "Audio File not found!";
          continue;
        }
        const KArchiveFile *f = static_cast<const KArchiveFile *>(e);
        if (!f->copyTo(serviceDir.path()))
          qDebug() << "FILE COULDN'T BE CREATED!";

        QFileInfo audFile = serviceDir.path() + "/" + audioFile.fileName();

        qDebug() << audFile.filePath();

        realAudio = audFile.filePath();
      }

      addItem(item.value("name").toString(), item.value("type").toString(),
              realBackground,
              item.value("backgroundType").toString(),
              item.value("text").toStringList(), realAudio,
              item.value("font").toString(), item.value("fontSize").toInt(),
              item.value("slideNumber").toInt(), item.value("loop").toBool());
    }

    return true;

  }

  return false;
}

void ServiceItemModel::clearAll() {
  for (int i = m_items.size(); i >= 0; i--) {
    removeItem(i);
  }
  emit allRemoved();
}

bool ServiceItemModel::loadLastSaved() {
  QSettings settings;
  return load(settings.value("lastSaveFile").toUrl());
}
