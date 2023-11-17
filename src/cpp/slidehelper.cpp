#include "slidehelper.h"
#include "serviceitemmodel.h"
#include "slidemodel.h"

// #include <podofo/podofo.h>
#include <QDebug>

// using namespace PoDoFo;
SlideHelper::SlideHelper(QObject *parent)
  : Slide{parent}
{
  qDebug() << "Initializing slide";
}

SlideHelper::SlideHelper(const QString &text, const QString &audio,
                         const QString &imageBackground,
                         const QString &videoBackground,
                         const QString &horizontalTextAlignment,
                         const QString &verticalTextAlignment,
                         const QString &font,
                         const int &fontSize,
                         const int &imageCount,
                         const bool &isPlaying,
                         const QString &type,
                         QObject *parent)
: Slide(parent),
  m_isPlaying(isPlaying),
  m_slideIndex(0)
{
  setText(text);
  setAudio(audio),
  setImageBackground(imageBackground),
  setVideoBackground(videoBackground),
  setVerticalTextAlignment(verticalTextAlignment),
  setHorizontalTextAlignment(horizontalTextAlignment),
  setFont(font),
  setFontSize(fontSize),
  setImageCount(imageCount),
  setType(type),
  qDebug() << "Initializing slide with defaults";
}

bool SlideHelper::isPlaying() const
{
  return m_isPlaying;
}

int SlideHelper::slideIndex() const
{
  return m_slideIndex;
}

int SlideHelper::slideSize() const
{
  return m_slideSize;
}

bool SlideHelper::loop() const
{
  return m_loop;
}

void SlideHelper::chngSlide(QVariantMap item, int index, SlideObject *slideObject) {
  // qDebug() << "Here is the pointer to the slideObj" << slideObj;
  // qDebug() << "Here is the item" << item;
  slideObject->changeSlide(item, index);
}

void SlideHelper::changeSlide(QVariantMap item, int index)
{
  // QVariantMap serviceItem = serviceItemModel->getItem(item.value("serviceItemId").toInt());
  if (item.value("text").toString() != text())
    setText(item.value("text").toString());
  if (item.value("type").toString() != type())
    setType(item.value("type").toString());
  if (item.value("audio").toString() != audio())
    setAudio(item.value("audio").toString());
  if (item.value("imageBackground").toString() != imageBackground())
    setImageBackground(item.value("imageBackground").toString());
  if (item.value("videoBackground").toString() != videoBackground()) {
    qDebug() << "Setting VIDEOBACKGROUND to:" << item.value("videoBackground").toString();
    setVideoBackground(item.value("videoBackground").toString());
  }
  if (item.value("verticalTextAlignment").toString() != verticalTextAlignment())
    setVerticalTextAlignment(item.value("verticalTextAlignment").toString());
  if (item.value("horizontalTextAlignment").toString() != horizontalTextAlignment())
    setHorizontalTextAlignment(item.value("horizontalTextAlignment").toString());
  if (item.value("font").toString() != font())
    setFont(item.value("font").toString());
  if (item.value("fontSize").toInt() != fontSize())
    setFontSize(item.value("fontSize").toInt());

  if (loop() != item.value("loop").toBool()) {
    setLoop(item.value("loop").toBool());
    emit loopChanged(loop());
  }
  setImageCount(item.value("imageCount").toInt());
  setSlideIndex(item.value("slideIndex").toInt());
  qDebug() << "THIS IS THE INDEX OF THE SLIDE!";
  qDebug() << index;
  emit slideChanged(index);
  // m_slideSize = serviceItem.value("slideNumber").toInt();

  // emit slideSizeChanged(m_slideSize);
}

bool SlideHelper::next(QVariantMap nextItem, SlideModel *slideModel)
{
  // QVariantMap serviceItem = serviceItemModel->getItem(nextItem.value("serviceItemId").toInt());
  setText(nextItem.value("text").toString());
  setType(nextItem.value("type").toString());
  setAudio(nextItem.value("audio").toString());
  setImageBackground(nextItem.value("imageBackground").toString());
  setVideoBackground(nextItem.value("videoBackground").toString());
  setVerticalTextAlignment(nextItem.value("verticalTextAlignment").toString());
  setHorizontalTextAlignment(nextItem.value("horizontalTextAlignment").toString());
  setFont(nextItem.value("font").toString());
  setFontSize(nextItem.value("fontSize").toInt());
  setImageCount(nextItem.value("imageCount").toInt());
  setSlideIndex(nextItem.value("slideIndex").toInt());
  if (loop() != nextItem.value("loop").toBool()) {
    setLoop(nextItem.value("loop").toBool());
    emit loopChanged(loop());
  }
  // m_slideSize = serviceItem.value("slideNumber").toInt();


  // emit slideSizeChanged(m_slideSize);
  return false;
}

bool SlideHelper::previous(QVariantMap prevItem, SlideModel *slideModel)
{
  // QVariantMap serviceItem = serviceItemModel->getItem(prevItem.value("serviceItemId").toInt());
  setText(prevItem.value("text").toString());
  setType(prevItem.value("type").toString());
  setAudio(prevItem.value("audio").toString());
  setImageBackground(prevItem.value("imageBackground").toString());
  setVideoBackground(prevItem.value("videoBackground").toString());
  setVerticalTextAlignment(prevItem.value("verticalTextAlignment").toString());
  setHorizontalTextAlignment(prevItem.value("horizontalTextAlignment").toString());
  setFont(prevItem.value("font").toString());
  setFontSize(prevItem.value("fontSize").toInt());
  setImageCount(prevItem.value("imageCount").toInt());
  setSlideIndex(prevItem.value("slideIndex").toInt());
  if (loop() != prevItem.value("loop").toBool()) {
    setLoop(prevItem.value("loop").toBool());
    emit loopChanged(loop());
  }
  // m_slideSize = serviceItem.value("slideNumber").toInt();

  // emit slideSizeChanged(m_slideSize);
  return false;
}

bool SlideHelper::changeSlideIndex(int index)
{
  qDebug() << "Starting to change slide index.";
  qDebug() << "SlideHelper Index: " << slideIndex() << " SlideHelper Size: " << slideSize();
  // QStringList text = serviceItemId().value("text").toStringList();
  if (index > slideSize() - 1 || index < 0) {
    qDebug() << "index is invalid: " << index;
    return false;
  }

  // since the string list is 0 indexed m_slideIndex actually
  // maps to the next item. So the prev text is minus 2
  if (type() == "song") {
    int textIndex = index;
    qDebug() << textIndex;
    // qDebug() << text[textIndex];
    // setText(text[textIndex]);
    m_slideIndex = index;
    emit slideIndexChanged(m_slideIndex);
    return true;
  }

  if (type() == "presentation") {
    qDebug() << "prev slide index: " << slideIndex();
    m_slideIndex = index;
    qDebug() << "new slide index: " << slideIndex();
    emit slideIndexChanged(m_slideIndex);
    return true;
  }
  return false;
}

void SlideHelper::play()
{
  m_isPlaying = true;
  emit isPlayingChanged(m_isPlaying);
}

void SlideHelper::setLoop(bool loop)
{
  m_loop = loop;
  emit loopChanged(m_loop);
}

void SlideHelper::pause()
{
  m_isPlaying = false;
  emit isPlayingChanged(m_isPlaying);
}

void SlideHelper::playPause()
{
  m_isPlaying = !m_isPlaying;
  emit isPlayingChanged(m_isPlaying);
}
