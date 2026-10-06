// SPDX-License-Identifier: AGPL-3.0-or-later
// Actual original retained page plus one competing beam group, no raster claim.
import java.nio.file.Path;
import java.util.*;
import java.awt.geom.Line2D;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordBeamOrderProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  List<Object> controls=new ArrayList<>();
  for(double y:new double[]{190.,230.}) {
   Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
   Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),staff=call(sys,"getFirstStaff");
   Object group=((Collection<?>)call(sig,"vertexSet")).stream().filter(n->n.getClass().getSimpleName().equals("BeamGroupInter")).findFirst().get();
   Object oldBeam=((List<?>)call(group,"getBeams")).get(0),first=call(group,"getFirstChord"),stem=call(first,"getStem"),voice=call(first,"getVoice"),measure=call(first,"getMeasure");
   Object added=beam(new double[]{288,y,320,y},4.,null,false),other=make("sig.inter.BeamGroupInter");call(sig,"addVertex",added);call(sig,"addVertex",other);call(other,"addMember",added);
   Object relation=make("sig.relation.BeamStemRelation");call(relation,"setBeamPortion",c("sig.relation.BeamPortion").getField("LEFT").get(null));call(sig,"addEdge",added,stem,relation);
   Object rest=make("sig.inter.RestInter",physical(365,240,8,20,"VERTICAL",null),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,staff,0.),restChord=make("sig.inter.RestChordInter",.8);
   call(restChord,"setStaff",staff);call(sig,"addVertex",rest);call(sig,"addVertex",restChord);call(restChord,"addMember",rest);call(measure,"addInter",restChord);call(sig,"addEdge",oldBeam,rest,make("sig.relation.BeamRestRelation"));
   call(first,"invalidateCache");List<Object> ordered=new ArrayList<>();for(Object b:(List<?>)call(first,"getBeams")){Line2D m=(Line2D)call(b,"getMedian");ordered.add(obj("median",new double[]{m.getX1(),m.getY1(),m.getX2(),m.getY2()}));}
   Object selected=call(first,"getBeamGroup");call(first,"justAssignVoice",(Object)null);call(first,"setVoice",voice);
   Object restVoice=call(restChord,"getVoice");controls.add(obj("added_median_y",y,"selected_added_group",selected==other,"ordered_beams",ordered,"rest_voice",restVoice==null?null:call(restVoice,"getId")));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","two declared extra competing beam-group conditions on existing original page, tail-to-head getBeams/getBeamGroup selection and same-measure rest propagation; not automatic competing group recognition","controls",controls));System.exit(0);
 }
}
