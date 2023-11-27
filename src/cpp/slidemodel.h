#ifndef SLIDEMODEL_H
#define SLIDEMODEL_H

#include "serviceitem.h"
#include "slide.h"
#include <QAbstractListModel>
#include <qabstractitemmodel.h>
#include <qnamespace.h>
#include <qobjectdefs.h>
#include <qsize.h>
#include "cxx-qt-gen/slide_model.cxxqt.h"

class SlideModelCpp : public QAbstractListModel {
  Q_OBJECT

public:
  explicit SlideModelCpp(QObject *parent = nullptr);

  enum Roles {
    TypeRole = Qt::UserRole,
    TextRole,
    AudioRole,
    ImageBackgroundRole,
    VideoBackgroundRole,
    HorizontalTextAlignmentRole,
    VerticalTextAlignmentRole,
    FontRole,
    FontSizeRole,
    ServiceItemIdRole,
    SlideIndexRole,
    ImageCountRole,
    ActiveRole,
    SelectedRole,
    LoopRole,
    VidThumbnailRole
  };

  // Basic functionality:
  int rowCount(const QModelIndex &parent = QModelIndex()) const override;
  // int columnCount(const QModelIndex &parent = QModelIndex()) const override;
  QVariant data(const QModelIndex &index,
                int role = Qt::DisplayRole) const override;
  QHash<int, QByteArray> roleNames() const override;

  // Q_INVOKABLE int index(int row, int column,
  //                       const QModelIndex &parent = QModelIndex()) const override;
  // Q_INVOKABLE QModelIndex parent(const QModelIndex &index) const override;

  // Editable:
  bool setData(const QModelIndex &index, const QVariant &value,
               int role = Qt::EditRole) override;
  Qt::ItemFlags flags(const QModelIndex &index) const override;

  // Helper methods
  void addItem(Slide *item);
  void insertItem(const int &index, Slide *item);
  Q_INVOKABLE void addItem(const QString &text, const QString &type,
                           const QString &imageBackground,
                           const QString &videoBackground,
                           const QString &audio,
                           const QString &font, const int &fontSize,
                           const QString &horizontalTextAlignment,
                           const QString &verticalTextAlignment,
                           const int &serviceItemId,
                           const int &slideIndex,
                           const int &imageCount,
                           const bool &loop);
  Q_INVOKABLE void insertItem(const int &index, const QString &text,
                              const QString &type, const QString &imageBackground,
                              const QString &videoBackground,
                              const QString &audio, const QString &font,
                              const int &fontSize,
                              const QString &horizontalTextAlignment,
                              const QString &verticalTextAlignment,
                              const int &serviceItemId,
                              const int &slideIndex,
                              const int &imageCount,
                              const bool &loop);
  Q_INVOKABLE void removeItem(int index);
  Q_INVOKABLE void removeItems();
  Q_INVOKABLE bool moveRows(int sourceIndex, int destIndex, int count);
  Q_INVOKABLE bool moveDown(int index);
  Q_INVOKABLE bool moveUp(int index);
  Q_INVOKABLE QVariantMap getItem(int index) const;
  Q_INVOKABLE QVariantMap getItemRust(int index, SlideModel *slidemodel) const;
  Q_INVOKABLE QVariantList getItems();
  Q_INVOKABLE int findSlideIdFromServItm(int index);
  Q_INVOKABLE QString thumbnailVideo(QString video, int serviceItemId, int index);
  Q_INVOKABLE QString thumbnailVideoRust(QString video, int serviceItemId, int index, SlideModel *slideModel);
  QImage frameToImage(const QString &video, int width);


public slots:
  Q_INVOKABLE bool select(int id);
  Q_INVOKABLE bool activate(int id);
  Q_INVOKABLE bool deactivate(int id);
  Q_INVOKABLE void removeServiceItem(const int &index, const ServiceItem &item);
  Q_INVOKABLE void clearAll();
  void addItemFromService(const int &index, const ServiceItem &item);
  void insertItemFromService(const int &index, const ServiceItem &item);
  void moveRowFromService(const int &fromIndex, const int &toIndex, const ServiceItem &item);

private:
  QList<Slide *> m_items;
};

#endif // SLIDEMODEL_H
