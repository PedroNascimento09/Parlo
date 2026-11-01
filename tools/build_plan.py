import re
import sys
from pathlib import Path

SOURCES = "docs/plan/v12_3"
DOCX_OUT = "Parlo_Plano_Mestre_v12_3_FINAL.docx"
PDF_OUT = "Parlo_Plano_Mestre_v12_3_FINAL.pdf"
TITLE = "PARLO"
HEADER = "Parlo · Plano Mestre v12.3"

ACCENT = (0x1F, 0x3A, 0x4D)
MUTED = (0x5B, 0x6B, 0x77)
RULE = (0xC8, 0xD2, 0xD9)
FILL = (0xEF, 0xF3, 0xF6)

BOLD = re.compile(r"\*\*(.+?)\*\*")


def split_bold(text):
    parts = []
    cursor = 0
    for match in BOLD.finditer(text):
        if match.start() > cursor:
            parts.append((text[cursor : match.start()], False))
        parts.append((match.group(1), True))
        cursor = match.end()
    if cursor < len(text):
        parts.append((text[cursor:], False))
    return parts or [(text, False)]


def parse(paths):
    blocks = []
    for path in paths:
        for raw in Path(path).read_text(encoding="utf-8").splitlines():
            line = raw.rstrip()
            if line.startswith("|"):
                cells = [cell.strip() for cell in line.strip("|").split("|")]
                if all(re.fullmatch(r":?-{2,}:?", cell) for cell in cells if cell):
                    continue
                if blocks and blocks[-1][0] == "table":
                    blocks[-1][1].append(cells)
                else:
                    blocks.append(("table", [cells]))
                continue
            stripped = line.strip()
            if not stripped:
                continue
            if stripped == "%%PAGEBREAK":
                blocks.append(("pagebreak", None))
            elif stripped.startswith("%%C "):
                blocks.append(("cover_title", stripped[4:]))
            elif stripped.startswith("%%S "):
                blocks.append(("cover_sub", stripped[4:]))
            elif stripped.startswith("%%M "):
                blocks.append(("cover_meta", stripped[4:]))
            elif stripped.startswith("# "):
                blocks.append(("part", stripped[2:]))
            elif stripped.startswith("#### "):
                blocks.append(("h3", stripped[5:]))
            elif stripped.startswith("### "):
                blocks.append(("h2", stripped[4:]))
            elif stripped.startswith("## "):
                blocks.append(("h1", stripped[3:]))
            elif stripped.startswith("> "):
                text = stripped[2:]
                if blocks and blocks[-1][0] == "callout":
                    blocks[-1][1].append(text)
                else:
                    blocks.append(("callout", [text]))
            elif stripped.startswith("- "):
                blocks.append(("bullet", stripped[2:]))
            else:
                blocks.append(("para", stripped))
    return blocks


def source_files():
    root = Path(SOURCES)
    files = sorted(root.glob("*.md"))
    if not files:
        raise SystemExit(f"no sources under {SOURCES}")
    return files


def build_docx(blocks):
    from docx import Document
    from docx.enum.section import WD_SECTION
    from docx.enum.table import WD_TABLE_ALIGNMENT
    from docx.enum.text import WD_ALIGN_PARAGRAPH, WD_BREAK
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Cm, Pt, RGBColor

    document = Document()
    section = document.sections[0]
    section.page_height = Cm(29.7)
    section.page_width = Cm(21.0)
    section.left_margin = Cm(2.3)
    section.right_margin = Cm(2.3)
    section.top_margin = Cm(2.2)
    section.bottom_margin = Cm(2.0)
    section.header_distance = Cm(1.1)
    section.footer_distance = Cm(1.1)

    normal = document.styles["Normal"]
    normal.font.name = "Calibri"
    normal.font.size = Pt(10.5)
    normal.paragraph_format.space_after = Pt(6)
    normal.paragraph_format.line_spacing = 1.12
    normal.element.rPr.rFonts.set(qn("w:eastAsia"), "Calibri")

    def shade(cell, colour):
        properties = cell._tc.get_or_add_tcPr()
        element = OxmlElement("w:shd")
        element.set(qn("w:val"), "clear")
        element.set(qn("w:fill"), colour)
        properties.append(element)

    def bottom_rule(paragraph, colour, size):
        properties = paragraph._p.get_or_add_pPr()
        borders = OxmlElement("w:pBdr")
        bottom = OxmlElement("w:bottom")
        bottom.set(qn("w:val"), "single")
        bottom.set(qn("w:sz"), str(size))
        bottom.set(qn("w:space"), "6")
        bottom.set(qn("w:color"), colour)
        borders.append(bottom)
        properties.append(borders)

    def add_runs(paragraph, text, size, colour=None, italic=False):
        for chunk, bold in split_bold(text):
            run = paragraph.add_run(chunk)
            run.bold = bold
            run.italic = italic
            run.font.size = Pt(size)
            run.font.name = "Calibri"
            if colour:
                run.font.color.rgb = RGBColor(*colour)

    def field(paragraph, instruction):
        run = paragraph.add_run()
        begin = OxmlElement("w:fldChar")
        begin.set(qn("w:fldCharType"), "begin")
        instr = OxmlElement("w:instrText")
        instr.set(qn("xml:space"), "preserve")
        instr.text = instruction
        separate = OxmlElement("w:fldChar")
        separate.set(qn("w:fldCharType"), "separate")
        keep = OxmlElement("w:t")
        keep.text = "1"
        end = OxmlElement("w:fldChar")
        end.set(qn("w:fldCharType"), "end")
        for element in (begin, instr, separate, keep, end):
            run._r.append(element)

    header = section.header.paragraphs[0]
    header.text = ""
    header.alignment = WD_ALIGN_PARAGRAPH.RIGHT
    add_runs(header, HEADER, 8, MUTED)
    bottom_rule(header, "C8D2D9", 4)

    footer = section.footer.paragraphs[0]
    footer.alignment = WD_ALIGN_PARAGRAPH.CENTER
    add_runs(footer, "", 8, MUTED)
    field(footer, "PAGE")
    for run in footer.runs:
        run.font.size = Pt(8)
        run.font.color.rgb = RGBColor(*MUTED)

    first_part = True
    for kind, payload in blocks:
        if kind == "cover_title":
            paragraph = document.add_paragraph()
            paragraph.alignment = WD_ALIGN_PARAGRAPH.CENTER
            paragraph.paragraph_format.space_before = Pt(150)
            run = paragraph.add_run(payload)
            run.bold = True
            run.font.size = Pt(46)
            run.font.color.rgb = RGBColor(*ACCENT)
            run.font.name = "Calibri"
        elif kind == "cover_sub":
            paragraph = document.add_paragraph()
            paragraph.alignment = WD_ALIGN_PARAGRAPH.CENTER
            paragraph.paragraph_format.space_before = Pt(10)
            add_runs(paragraph, payload, 15, ACCENT)
        elif kind == "cover_meta":
            paragraph = document.add_paragraph()
            paragraph.alignment = WD_ALIGN_PARAGRAPH.CENTER
            paragraph.paragraph_format.space_before = Pt(4)
            add_runs(paragraph, payload, 10, MUTED)
        elif kind == "pagebreak":
            document.add_paragraph().add_run().add_break(WD_BREAK.PAGE)
        elif kind == "part":
            paragraph = document.add_paragraph()
            if not first_part:
                paragraph.paragraph_format.page_break_before = True
            first_part = False
            paragraph.paragraph_format.space_before = Pt(0 if not first_part else 0)
            paragraph.paragraph_format.space_after = Pt(16)
            add_runs(paragraph, payload, 22, ACCENT)
            bottom_rule(paragraph, "1F3A4D", 12)
            outline = paragraph._p.get_or_add_pPr()
            level = OxmlElement("w:outlineLvl")
            level.set(qn("w:val"), "0")
            outline.append(level)
        elif kind == "h1":
            paragraph = document.add_paragraph()
            paragraph.paragraph_format.space_before = Pt(16)
            paragraph.paragraph_format.space_after = Pt(8)
            paragraph.paragraph_format.keep_with_next = True
            add_runs(paragraph, payload, 15, ACCENT)
            outline = paragraph._p.get_or_add_pPr()
            level = OxmlElement("w:outlineLvl")
            level.set(qn("w:val"), "1")
            outline.append(level)
        elif kind == "h2":
            paragraph = document.add_paragraph()
            paragraph.paragraph_format.space_before = Pt(12)
            paragraph.paragraph_format.space_after = Pt(4)
            paragraph.paragraph_format.keep_with_next = True
            add_runs(paragraph, payload, 12, ACCENT)
            outline = paragraph._p.get_or_add_pPr()
            level = OxmlElement("w:outlineLvl")
            level.set(qn("w:val"), "2")
            outline.append(level)
        elif kind == "h3":
            paragraph = document.add_paragraph()
            paragraph.paragraph_format.space_before = Pt(10)
            paragraph.paragraph_format.space_after = Pt(3)
            paragraph.paragraph_format.keep_with_next = True
            add_runs(paragraph, payload, 10.5, MUTED)
        elif kind == "para":
            paragraph = document.add_paragraph()
            paragraph.alignment = WD_ALIGN_PARAGRAPH.JUSTIFY
            add_runs(paragraph, payload, 10.5)
        elif kind == "bullet":
            paragraph = document.add_paragraph(style="List Bullet")
            paragraph.paragraph_format.space_after = Pt(2)
            paragraph.paragraph_format.left_indent = Cm(0.7)
            add_runs(paragraph, payload, 10.5)
        elif kind == "callout":
            text = " ".join(payload)
            table = document.add_table(rows=1, cols=1)
            table.alignment = WD_TABLE_ALIGNMENT.CENTER
            table.autofit = True
            cell = table.rows[0].cells[0]
            shade(cell, "EFF3F6")
            paragraph = cell.paragraphs[0]
            paragraph.paragraph_format.space_before = Pt(4)
            paragraph.paragraph_format.space_after = Pt(4)
            add_runs(paragraph, text, 10, ACCENT)
            document.add_paragraph().paragraph_format.space_after = Pt(2)
        elif kind == "table":
            rows = payload
            columns = max(len(row) for row in rows)
            table = document.add_table(rows=len(rows), cols=columns)
            table.style = "Table Grid"
            table.alignment = WD_TABLE_ALIGNMENT.CENTER
            for index, row in enumerate(rows):
                for column in range(columns):
                    cell = table.rows[index].cells[column]
                    cell.text = ""
                    paragraph = cell.paragraphs[0]
                    paragraph.paragraph_format.space_after = Pt(1)
                    text = row[column] if column < len(row) else ""
                    add_runs(paragraph, text, 8.5, ACCENT if index == 0 else None)
                    for run in paragraph.runs:
                        if index == 0:
                            run.bold = True
                    if index == 0:
                        shade(cell, "EFF3F6")
            document.add_paragraph().paragraph_format.space_after = Pt(2)

    document.save(DOCX_OUT)
    return DOCX_OUT


def build_pdf(blocks):
    from reportlab.lib import colors
    from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
    from reportlab.lib.pagesizes import A4
    from reportlab.lib.styles import ParagraphStyle
    from reportlab.lib.units import cm
    from reportlab.pdfbase import pdfmetrics
    from reportlab.pdfbase.ttfonts import TTFont
    from reportlab.platypus import (
        BaseDocTemplate,
        Frame,
        KeepTogether,
        PageTemplate,
        Paragraph,
        Spacer,
        Table,
        TableStyle,
    )

    fonts = {
        "body": ("C:/Windows/Fonts/calibri.ttf", "C:/Windows/Fonts/calibrib.ttf", "C:/Windows/Fonts/calibrii.ttf"),
        "alt": ("C:/Windows/Fonts/arial.ttf", "C:/Windows/Fonts/arialbd.ttf", "C:/Windows/Fonts/ariali.ttf"),
    }
    family = None
    for name, (regular, bold, italic) in fonts.items():
        if all(Path(path).exists() for path in (regular, bold, italic)):
            pdfmetrics.registerFont(TTFont("Parlo", regular))
            pdfmetrics.registerFont(TTFont("Parlo-Bold", bold))
            pdfmetrics.registerFont(TTFont("Parlo-Italic", italic))
            pdfmetrics.registerFontFamily("Parlo", normal="Parlo", bold="Parlo-Bold", italic="Parlo-Italic")
            family = "Parlo"
            break
    if family is None:
        raise SystemExit("no usable system font for the PDF")

    accent = colors.Color(ACCENT[0] / 255, ACCENT[1] / 255, ACCENT[2] / 255)
    muted = colors.Color(MUTED[0] / 255, MUTED[1] / 255, MUTED[2] / 255)
    rule = colors.Color(RULE[0] / 255, RULE[1] / 255, RULE[2] / 255)
    fill = colors.Color(FILL[0] / 255, FILL[1] / 255, FILL[2] / 255)

    styles = {
        "cover_title": ParagraphStyle("cover_title", fontName="Parlo-Bold", fontSize=44, leading=50, alignment=TA_CENTER, textColor=accent),
        "cover_sub": ParagraphStyle("cover_sub", fontName="Parlo", fontSize=15, leading=21, alignment=TA_CENTER, textColor=accent, spaceBefore=10),
        "cover_meta": ParagraphStyle("cover_meta", fontName="Parlo", fontSize=10, leading=15, alignment=TA_CENTER, textColor=muted, spaceBefore=4),
        "part": ParagraphStyle("part", fontName="Parlo-Bold", fontSize=21, leading=26, textColor=accent, spaceAfter=4, spaceBefore=0),
        "h1": ParagraphStyle("h1", fontName="Parlo-Bold", fontSize=14.5, leading=19, textColor=accent, spaceBefore=14, spaceAfter=6),
        "h2": ParagraphStyle("h2", fontName="Parlo-Bold", fontSize=11.5, leading=16, textColor=accent, spaceBefore=10, spaceAfter=3),
        "h3": ParagraphStyle("h3", fontName="Parlo-Bold", fontSize=10, leading=14, textColor=muted, spaceBefore=8, spaceAfter=2),
        "body": ParagraphStyle("body", fontName="Parlo", fontSize=10, leading=14.4, alignment=TA_JUSTIFY, spaceAfter=6),
        "bullet": ParagraphStyle("bullet", fontName="Parlo", fontSize=10, leading=14, alignment=TA_LEFT, leftIndent=14, bulletIndent=4, spaceAfter=2.5),
        "callout": ParagraphStyle("callout", fontName="Parlo", fontSize=9.6, leading=13.6, textColor=accent),
        "cell": ParagraphStyle("cell", fontName="Parlo", fontSize=8, leading=10.6, alignment=TA_LEFT),
        "cell_head": ParagraphStyle("cell_head", fontName="Parlo-Bold", fontSize=8, leading=10.6, textColor=accent),
    }

    def markup(text):
        return BOLD.sub(r"<b>\1</b>", text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"))

    width = A4[0] - 4.6 * cm

    def draw_page(canvas, document):
        canvas.saveState()
        canvas.setFont("Parlo", 8)
        canvas.setFillColor(muted)
        canvas.drawRightString(A4[0] - 2.3 * cm, A4[1] - 1.5 * cm, HEADER)
        canvas.setStrokeColor(rule)
        canvas.setLineWidth(0.4)
        canvas.line(2.3 * cm, A4[1] - 1.65 * cm, A4[0] - 2.3 * cm, A4[1] - 1.65 * cm)
        canvas.drawCentredString(A4[0] / 2, 1.3 * cm, str(document.page))
        canvas.restoreState()

    document = BaseDocTemplate(
        PDF_OUT,
        pagesize=A4,
        leftMargin=2.3 * cm,
        rightMargin=2.3 * cm,
        topMargin=2.3 * cm,
        bottomMargin=2.0 * cm,
        title="Parlo — Plano Mestre v12.3",
        author="Pedro Nascimento",
        subject="Plano mestre técnico, produto e open source do Parlo",
    )
    frame = Frame(document.leftMargin, document.bottomMargin, width, A4[1] - 4.3 * cm, id="body")
    document.addPageTemplates([PageTemplate(id="main", frames=[frame], onPage=draw_page)])

    story = []
    for kind, payload in blocks:
        if kind == "cover_title":
            story.append(Spacer(1, 5.2 * cm))
            story.append(Paragraph(markup(payload), styles["cover_title"]))
        elif kind == "cover_sub":
            story.append(Paragraph(markup(payload), styles["cover_sub"]))
        elif kind == "cover_meta":
            story.append(Paragraph(markup(payload), styles["cover_meta"]))
        elif kind == "pagebreak":
            from reportlab.platypus import PageBreak

            story.append(PageBreak())
        elif kind == "part":
            from reportlab.platypus import PageBreak

            if story:
                story.append(PageBreak())
            story.append(Paragraph(markup(payload), styles["part"]))
            story.append(Table([[""]], colWidths=[width], rowHeights=[2], style=TableStyle([("BACKGROUND", (0, 0), (-1, -1), accent)])))
            story.append(Spacer(1, 12))
        elif kind == "h1":
            story.append(Paragraph(markup(payload), styles["h1"]))
        elif kind == "h2":
            story.append(Paragraph(markup(payload), styles["h2"]))
        elif kind == "h3":
            story.append(Paragraph(markup(payload), styles["h3"]))
        elif kind == "para":
            story.append(Paragraph(markup(payload), styles["body"]))
        elif kind == "bullet":
            story.append(Paragraph(markup(payload), styles["bullet"], bulletText="•"))
        elif kind == "callout":
            inner = Paragraph(markup(" ".join(payload)), styles["callout"])
            table = Table([[inner]], colWidths=[width])
            table.setStyle(
                TableStyle(
                    [
                        ("BACKGROUND", (0, 0), (-1, -1), fill),
                        ("BOX", (0, 0), (-1, -1), 0.4, rule),
                        ("LEFTPADDING", (0, 0), (-1, -1), 8),
                        ("RIGHTPADDING", (0, 0), (-1, -1), 8),
                        ("TOPPADDING", (0, 0), (-1, -1), 6),
                        ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
                    ]
                )
            )
            story.append(table)
            story.append(Spacer(1, 7))
        elif kind == "table":
            rows = payload
            columns = max(len(row) for row in rows)
            weights = []
            for column in range(columns):
                longest = max(len(row[column]) if column < len(row) else 0 for row in rows)
                weights.append(max(longest, 6))
            total = sum(weights)
            widths = [width * weight / total for weight in weights]
            data = []
            for index, row in enumerate(rows):
                cells = []
                for column in range(columns):
                    text = row[column] if column < len(row) else ""
                    style = styles["cell_head"] if index == 0 else styles["cell"]
                    cells.append(Paragraph(markup(text), style))
                data.append(cells)
            table = Table(data, colWidths=widths, repeatRows=1)
            table.setStyle(
                TableStyle(
                    [
                        ("GRID", (0, 0), (-1, -1), 0.35, rule),
                        ("BACKGROUND", (0, 0), (-1, 0), fill),
                        ("VALIGN", (0, 0), (-1, -1), "TOP"),
                        ("LEFTPADDING", (0, 0), (-1, -1), 4),
                        ("RIGHTPADDING", (0, 0), (-1, -1), 4),
                        ("TOPPADDING", (0, 0), (-1, -1), 3),
                        ("BOTTOMPADDING", (0, 0), (-1, -1), 3),
                    ]
                )
            )
            story.append(KeepTogether(table) if len(rows) <= 12 else table)
            story.append(Spacer(1, 7))

    document.build(story)
    return PDF_OUT


def main() -> int:
    blocks = parse(source_files())
    if not blocks:
        raise SystemExit("nothing to build")
    docx_path = build_docx(blocks)
    pdf_path = build_pdf(blocks)
    counts = {}
    for kind, _ in blocks:
        counts[kind] = counts.get(kind, 0) + 1
    print(f"blocks: {counts}")
    print(f"docx: {docx_path}")
    print(f"pdf: {pdf_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
