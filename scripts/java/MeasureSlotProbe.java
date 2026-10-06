// SPDX-License-Identifier: AGPL-3.0-or-later
// Invoke original SlotsRetriever after resetRhythm. Saved image states and
// declared method controls remain separate; no recognition rerun or XML reuse.
import java.nio.file.Path;import java.util.*;import java.awt.Rectangle;
import com.fasterxml.jackson.databind.ObjectMapper;
public class MeasureSlotProbe extends FoundationOverlapProbe {
 static Object load(String p)throws Exception {Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(p));return call(((List<?>)call(book,"getStubs")).get(0),"getSheet");}
 static Object inspect(Object measure)throws Exception {
  call(measure,"resetRhythm");Object stack=call(measure,"getStack"),sys=call(stack,"getSystem"),sig=call(sys,"getSig");
  java.util.List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));chords.removeIf(ch->{try{return (Boolean)call(ch,"isMeasureRest");}catch(Exception e){throw new RuntimeException(e);}});
  java.util.List<Object> facts=new ArrayList<>(),explicit=new ArrayList<>(),mirrors=new ArrayList<>(),alignments=new ArrayList<>(),groups=new ArrayList<>();
  for(Object ch:chords) {
   Object stem=call(ch,"getStem");java.util.List<Object> pitches=new ArrayList<>(),notes=new ArrayList<>();
   for(Object n:(java.util.List<?>)call(ch,"getNotes")){pitches.add(call(n,"getIntegerPitch"));notes.add(n);}
   java.util.List<Object> mirrorPartners=new ArrayList<>();
   for(Object other:chords)if(other!=ch)for(Object n:notes){Object m=call(n,"getMirror");if(m!=null&&((java.util.List<?>)call(other,"getNotes")).contains(m)){mirrorPartners.add(chords.indexOf(other));break;}}
   facts.add(obj("id",chords.indexOf(ch),"bounds",rect(call(ch,"getBounds")),"dot_bounds",rect(call(ch,"getBoundsWithDots")),"x",call(stack,"getXOffset",call(ch,"getCenter")),"head",point(call(ch,"getHeadLocation")),"rest",call(ch,"isRest"),"stem",stem==null?null:call(stem,"getId"),"stem_dir",call(ch,"getStemDir"),"has_beams",!((java.util.List<?>)call(ch,"getBeams")).isEmpty(),"pitches",pitches,"leading_pitch",call(call(ch,"getLeadingNote"),"getIntegerPitch"),"time_offset",null));
   for(Object other:mirrorPartners)if(chords.indexOf(ch)<((Number)other).intValue())mirrors.add(java.util.List.of(chords.indexOf(ch),other));
  }
  for(Object e:(Collection<?>)call(sig,"edgeSet")) {
   String kind=e.getClass().getSimpleName();Object a=call(sig,"getEdgeSource",e),b=call(sig,"getEdgeTarget",e);
   if(java.util.List.of("SameTimeRelation","SeparateTimeRelation","NextInVoiceRelation").contains(kind)&&chords.contains(a)&&chords.contains(b))explicit.add(obj("kind",kind,"a",chords.indexOf(a),"b",chords.indexOf(b)));
   if(kind.equals("StemAlignmentRelation"))for(int i=0;i<chords.size();i++)for(int j=i+1;j<chords.size();j++)if((call(chords.get(i),"getStem")==a&&call(chords.get(j),"getStem")==b)||(call(chords.get(i),"getStem")==b&&call(chords.get(j),"getStem")==a))alignments.add(java.util.List.of(i,j));
  }
  for(Object group:(Collection<?>)call(measure,"getBeamGroups")){boolean small=false;for(Object b:(java.util.List<?>)call(group,"getMembers"))if(b.getClass().getSimpleName().equals("SmallBeamInter"))small=true;if(!small){java.util.List<Object> row=new ArrayList<>();for(Object ch:(java.util.List<?>)call(group,"getChords"))if(chords.contains(ch))row.add(chords.indexOf(ch));groups.add(row);}}
  Map<String,Object> output=obj("number",call(stack,"getIdValue"),"system",call(sys,"getId"),"chords",facts,"explicit",explicit,"mirrors",mirrors,"aligned",alignments,"groups",groups,"interline",call(call(call(sys,"getSheet"),"getScale"),"getInterline"));
  for(boolean wide:new boolean[]{false,true}) {
   Object retriever=make("sheet.rhythm.SlotsRetriever",measure,wide);java.util.List<Object> relations=new ArrayList<>(),slots=new ArrayList<>();
   for(Object a:chords)for(Object b:chords)if(a!=b){Object rel=call(retriever,"getRel",a,b);relations.add(obj("a",chords.indexOf(a),"b",chords.indexOf(b),"rel",rel==null?null:((Enum<?>)rel).name()));}
   for(Object slot:(java.util.List<?>)call(retriever,"buildSlots")){java.util.List<Object> ids=new ArrayList<>();for(Object ch:(java.util.List<?>)call(slot,"getChords"))ids.add(chords.indexOf(ch));slots.add(obj("id",call(slot,"getId"),"chord_ids",ids));}
   output.put(wide?"wide":"narrow",obj("relations",relations,"slots",slots));
  }
  return output;
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);java.util.List<Object> pages=new ArrayList<>();
  for(int i=1;i<args.length;i++){Object sheet=load(args[i]);java.util.List<Object> measures=new ArrayList<>();for(Object sys:(java.util.List<?>)call(sheet,"getSystems"))for(Object stack:(java.util.List<?>)call(sys,"getStacks"))measures.add(inspect(call(stack,"getFirstMeasure")));pages.add(obj("input",Path.of(args[i]).getFileName().toString(),"measures",measures));}
  java.util.List<Object> controls=new ArrayList<>();
  for(String name:new String[]{"close-chain","separate-close","same-with-intermediate","adjacent-opposite","aligned-stems"}) {
   Object sheet=load(args[1]),sys=((java.util.List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),measure=call(((java.util.List<?>)call(sys,"getStacks")).get(0),"getFirstMeasure");
   for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(java.util.List.of("HeadInter","HeadChordInter","RestInter","RestChordInter","StemInter","BeamInter","BeamHookInter","BeamGroupInter","FlagInter").contains(n.getClass().getSimpleName())&&!(Boolean)call(n,"isRemoved"))call(n,"remove");
   java.util.List<Object> chords=new ArrayList<>(),stems=new ArrayList<>();int count=name.equals("adjacent-opposite")||name.equals("aligned-stems")?2:3;
   for(int i=0;i<count;i++) {
    boolean stemmed=name.equals("adjacent-opposite")||name.equals("aligned-stems");int x=300+i*6,y=190+i*20;if(stemmed)y=190;
    Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));call(sig,"addVertex",chord);
    Object head=make("sig.inter.HeadInter",new Rectangle(x,y,12,8),c("glyph.Shape").getField(stemmed?"NOTEHEAD_BLACK":"WHOLE_NOTE").get(null),.8,call(sys,"getFirstStaff"),(double)i*4);call(sig,"addVertex",head);call(chord,"addMember",head);
    if(stemmed){Object stem=make("sig.inter.StemInter",physical(x+10,i==0?160:194,2,40,"VERTICAL",null),.8);call(sig,"addVertex",stem);call(chord,"setStem",stem);stems.add(stem);}
    call(measure,"addInter",chord);chords.add(chord);
   }
   if(name.equals("separate-close"))call(sig,"addEdge",chords.get(0),chords.get(1),make("sig.relation.SeparateTimeRelation"));
   if(name.equals("same-with-intermediate"))call(sig,"addEdge",chords.get(0),chords.get(2),make("sig.relation.SameTimeRelation"));
   if(name.equals("aligned-stems"))call(sig,"addEdge",stems.get(0),stems.get(1),make("sig.relation.StemAlignmentRelation"));
   controls.add(obj("name",name,"observation",inspect(measure)));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","original reset+narrow/wide SlotsRetriever on saved successful image states; separate five declared method controls, not general voice matching or new image admission","pages",pages,"controls",controls));System.exit(0);
 }
}
