
add_executable(lumina main.cpp resources.qrc)

target_sources(lumina
 PRIVATE
 main.cpp resources.qrc
 cpp/songsqlmodel.cpp cpp/songsqlmodel.h
 cpp/serviceitemmodel.cpp cpp/serviceitemmodel.h
 cpp/serviceitem.cpp cpp/serviceitem.h
 cpp/slideobject.cpp cpp/slideobject.h
 cpp/slidemodel.cpp cpp/slidemodel.h
 cpp/slide.cpp cpp/slide.h
 cpp/videosqlmodel.cpp cpp/videosqlmodel.h
 cpp/imagesqlmodel.cpp cpp/imagesqlmodel.h
 cpp/filemanager.cpp cpp/filemanager.h
 cpp/presentationsqlmodel.cpp cpp/presentationsqlmodel.h
 cpp/mpv/mpvobject.h cpp/mpv/mpvobject.cpp
 cpp/mpv/qthelper.hpp cpp/mpv/mpvhelpers.h
 cpp/framedecoder.h cpp/framedecoder.cpp
)

target_compile_options (lumina PUBLIC -fexceptions)

target_link_libraries(lumina PRIVATE
    ${AVUTIL_LIBRARIES}
    ${AVFILTER_LIBRARIES}
    ${AVFORMAT_LIBRARIES}
    ${AVCODEC_LIBRARIES}
    ${SWSCALE_LIBRARIES}
)
