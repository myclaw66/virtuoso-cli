; RB_connectTerminal - Place wire stub + net label at an instance terminal
; Used by vcli schematic build for non-interactive schematic connections
;
; Parameters:
;   cv       - cellView object (must be open for editing)
;   instName - instance name (e.g. "M5")
;   termName - terminal name (e.g. "D", "G", "S", "B")
;   netName  - net label name (e.g. "vtail", "vdd!")
;
; Creates a short wire path at the terminal pin position and labels it
; with the net name. Terminals sharing the same label are connected.

procedure(RB_connectTerminal(cv instName termName netName)
  let((inst term pinFig pinBox center wire net labelId stubEnd)
    inst = car(setof(i cv~>instances strcmp(i~>name instName)==0))
    unless(inst
      printf("  [ERROR] Instance %s not found\n" instName)
      return(nil))
    term = dbFindTermByName(inst~>master termName)
    unless(term
      printf("  [ERROR] Terminal %s not found on %s\n" termName instName)
      return(nil))
    ; Get pin position in cellview coordinates
    pinFig = car(car(term~>pins)~>figs)
    pinBox = dbTransformBBox(pinFig~>bBox inst~>transform)
    center = list(
      quotient(plus(xCoord(lowerLeft(pinBox)) xCoord(upperRight(pinBox))) 2.0)
      quotient(plus(yCoord(lowerLeft(pinBox)) yCoord(upperRight(pinBox))) 2.0)
    )
    ; Determine stub direction based on pin position relative to instance origin
    ; Gate (at x=0 in symbol) → extend left; D/S/B (at x=0.25) → extend right
    if(xCoord(lowerLeft(pinBox)) < car(inst~>xy) + 0.1
      then stubEnd = list(car(center) - 0.25  cadr(center))  ; left stub for Gate
      else stubEnd = list(car(center) + 0.25  cadr(center))  ; right stub for D/S/B
    )
    ; Create wire path on "wire" layer
    wire = dbCreatePath(cv list("wire" "drawing") list(center stubEnd) 0.0625)
    ; Create net label
    labelId = dbCreateLabel(cv list("wire" "label") stubEnd netName
                "lowerLeft" "R0" "stick" 0.0625)
    ; Associate label with wire
    when(wire && labelId
      dbAddFigToFigGroup(wire labelId))
    t
  )
)
