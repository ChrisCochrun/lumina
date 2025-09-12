use std::path::Path;

use cosmic::iced::ContentFit;
use cosmic::iced::Length;
use cosmic::widget::image;
use cosmic::widget::image::Handle;
use cosmic::Element;
use miette::IntoDiagnostic;
use miette::Result;
use miette::Severity;
use mupdf::Colorspace;
use mupdf::Document;
use mupdf::Matrix;
use mupdf::Page;

#[derive(Debug, Clone)]
pub struct PdfViewer {
    document: Option<Document>,
    pages: Option<Vec<Page>>,
    current_page: Option<Page>,
    current_index: usize,
    handle: Option<Handle>,
}

enum Message {}

impl PdfViewer {
    pub fn with_pdf(pdf: impl AsRef<Path>) -> Result<Self> {
        let pdf_path = pdf.as_ref();
        let document = Document::open(pdf_path).into_diagnostic()?;
        let pages = document.pages().into_diagnostic()?;
        let pages: Vec<Page> =
            pages.filter_map(|page| page.ok()).collect();
        let Some(page) = document.pages().into_diagnostic()?.next()
        else {
            return Err(miette::miette!(
                severity = Severity::Warning,
                "There isn't a first page here"
            ));
        };
        let page = page.into_diagnostic()?;
        let matrix = Matrix::IDENTITY;
        let colorspace = Colorspace::device_rgb();
        let pixmap = page
            .to_pixmap(&matrix, &colorspace, true, true)
            .into_diagnostic()?;
        let handle = pixmap.samples().to_vec();
        Ok(Self {
            document: Some(document),
            pages: Some(pages),
            current_page: Some(page),
            current_index: 0,
            handle: Some(Handle::from_bytes(handle)),
        })
    }

    pub fn insert_pdf(
        &mut self,
        pdf: impl AsRef<Path>,
    ) -> Result<()> {
        let pdf_path = pdf.as_ref();
        let document = Document::open(pdf_path).into_diagnostic()?;
        let pages = document.pages().into_diagnostic()?;
        let pages: Vec<Page> =
            pages.filter_map(|page| page.ok()).collect();
        let Some(page) = document.pages().into_diagnostic()?.next()
        else {
            return Err(miette::miette!(
                severity = Severity::Warning,
                "There isn't a first page here"
            ));
        };
        let page = page.into_diagnostic()?;
        let matrix = Matrix::IDENTITY;
        let colorspace = Colorspace::device_rgb();
        let pixmap = page
            .to_pixmap(&matrix, &colorspace, true, true)
            .into_diagnostic()?;
        let handle = pixmap.samples().to_vec();
        self.document = Some(document);
        self.pages = Some(pages);
        self.current_page = Some(page);
        self.current_index = 0;
        self.handle = Some(Handle::from_bytes(handle));
        Ok(())
    }

    pub fn next_page(&mut self) -> Result<()> {
        let Some(ref pages) = self.pages else {
            return Err(miette::miette!("No pages in doc"));
        };
        let Some(page) = pages.get(self.current_index + 1) else {
            return Err(miette::miette!("There isn't a next page"));
        };
        let matrix = Matrix::IDENTITY;
        let colorspace = Colorspace::device_rgb();
        let pixmap = page
            .to_pixmap(&matrix, &colorspace, true, true)
            .into_diagnostic()?;
        let handle = pixmap.samples().to_vec();
        self.current_page = Some(page.to_owned());
        self.current_index += 1;
        self.handle = Some(Handle::from_bytes(handle));
        Ok(())
    }

    pub fn previous_page(&mut self) -> Result<()> {
        if self.current_index == 0 {
            return Err(miette::miette!("You are at the first page"));
        }
        let Some(ref pages) = self.pages else {
            return Err(miette::miette!("No pages in doc"));
        };
        let Some(page) = pages.get(self.current_index - 1) else {
            return Err(miette::miette!(
                "There isn't a previous page"
            ));
        };
        let matrix = Matrix::IDENTITY;
        let colorspace = Colorspace::device_rgb();
        let pixmap = page
            .to_pixmap(&matrix, &colorspace, true, true)
            .into_diagnostic()?;
        let handle = pixmap.samples().to_vec();
        self.current_page = Some(page.to_owned());
        self.current_index -= 1;
        self.handle = Some(Handle::from_bytes(handle));
        Ok(())
    }

    pub fn view(&self) -> Option<Element<Message>> {
        let Some(handle) = self.handle.clone() else {
            return None;
        };
        Some(
            image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Contain)
                .into(),
        )
    }
}
