import "pdfjs-dist/web/pdf_viewer.css"
import "./style.css"
import * as pdfjsLib from "pdfjs-dist"
import { PDFViewer, EventBus } from "pdfjs-dist/web/pdf_viewer.mjs"
import workerSrc from "pdfjs-dist/build/pdf.worker.min.mjs?url"

pdfjsLib.GlobalWorkerOptions.workerSrc = workerSrc

async function setup() {
  const notifications = new EventSource("/notifications")

  const container = document.getElementById("viewerContainer") as HTMLDivElement
  const eventBus = new EventBus()

  const pdfViewer = new PDFViewer({
    container,
    eventBus
  })

  eventBus.on("pagesinit", function() {
    pdfViewer.currentScaleValue = "auto"
  })

  window.addEventListener("resize", function(){
    pdfViewer.currentScaleValue = "auto"
  })

  async function load() {
    const pdf = await pdfjsLib.getDocument("/preview.pdf").promise
    pdfViewer.setDocument(pdf)
  }

  notifications.addEventListener("message", async function() {
    const [scrollTop, scrollLeft] = [container.scrollTop, container.scrollLeft]
    container.style.opacity = "0"
    await load()
    setTimeout(function() {
      container.scrollTo(scrollLeft, scrollTop)
      container.style.opacity = "1"
    }, 10)
  })

  notifications.addEventListener("open", load)

  await load()
}

window.addEventListener("DOMContentLoaded", setup)
