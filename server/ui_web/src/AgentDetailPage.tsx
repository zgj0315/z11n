import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { pdfjs, Document, Page } from "react-pdf";
import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";
import type { PDFDocumentProxy } from "pdfjs-dist";
import restful_api from "./RESTfulApi.tsx";

pdfjs.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url
).toString();

const App: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [numPages, setNumPages] = useState<number>();
  const [pageNumber, setPageNumber] = useState<number>(1);
  const [pdfData, setPdfData] = useState<Blob | null>(null);

  function onDocumentLoadSuccess({
    numPages: nextNumPages,
  }: PDFDocumentProxy): void {
    setNumPages(nextNumPages);
  }

  const goToPrevPage = () => {
    setPageNumber((prev) => Math.max(prev - 1, 1));
  };

  const goToNextPage = () => {
    if (numPages) {
      setPageNumber((prev) => Math.min(prev + 1, numPages));
    }
  };
  useEffect(() => {
    const fetchPdf = async () => {
      try {
        const response = await restful_api.get(`/api/pdf_articles/${id}`, {
          responseType: "blob",
        });
        console.log("Blob type:", response.data.type); // 调试 Blob 类型
        console.log("Blob size:", response.data.size); // 调试 Blob 大小
        setPdfData(response.data);
      } catch (error) {
        console.error("Failed to fetch PDF:", error);
      }
    };
    fetchPdf();
  }, [id]);
  return (
    <div>
      <Document file={pdfData} onLoadSuccess={onDocumentLoadSuccess}>
        <Page pageNumber={pageNumber} />
      </Document>
      <p>
        <button onClick={goToPrevPage} disabled={pageNumber <= 1}>
          上一页
        </button>
        Page {pageNumber} of {numPages}
        <button
          onClick={goToNextPage}
          disabled={!numPages || pageNumber >= numPages}
        >
          下一页
        </button>
      </p>
    </div>
  );
};

export default App;
