// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.geom.Line2D;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletBeatProbe extends BeamRestClearProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),last=call(old,"getLastChord"),offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);call(last,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> oldRelations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+oldRelations.size());oldRelations.add(r);}

            Object extraGroup=make("sig.inter.BeamGroupInter");call(sig,"addVertex",extraGroup);labels.put(extraGroup,"extra_group");
            double[] median={560.,190.,770.,190.};Object extraBeam=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(median[0],median[1],median[2],median[3]),4.);call(sig,"addVertex",extraBeam);labels.put(extraBeam,"extra_beam");call(extraGroup,"addMember",extraBeam);
            for(int i:List.of(5,7)){Object e=make("sig.relation.BeamStemRelation");call(sig,"addEdge",extraBeam,call(chords.get(i),"getStem"),e);call(chords.get(i),"invalidateCache");}
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),scenarios=new ArrayList<>();
            Object generator=make("sheet.rhythm.TupletGenerator",measure);Method build=c("sheet.rhythm.TupletGenerator").getDeclaredMethod("buildGroups",c("sheet.rhythm.Voice"),c("math.Rational"));build.setAccessible(true);Method check=c("sheet.rhythm.TupletGenerator").getDeclaredMethod("checkGroups",List.class);check.setAccessible(true);
            for(Object voice:voices){List<?> members=(List<?>)call(voice,"getChords");List<Object> saved=new ArrayList<>();for(Object ch:members)saved.add(call(ch,"getTimeOffset"));
                for(String pattern:List.of("original","all_zero","exact_threshold","large_gap","first_null","later_null"))for(String beatText:List.of("1/4","1/2","1/8","0")) {
                    List<Object> times=new ArrayList<>();for(int i=0;i<members.size();i++) {
                        Object time=saved.get(i);
                        if(pattern.equals("all_zero"))time=make("math.Rational",0,1);
                        if(pattern.equals("exact_threshold"))time=make("math.Rational",i*3,8);
                        if(pattern.equals("large_gap"))time=make("math.Rational",i==0?0:5,1);
                        if(pattern.equals("first_null")&&i==0)time=null;
                        if(pattern.equals("later_null")&&i==1)time=null;
                        call(members.get(i),"setTimeOffset",(Object)time);times.add(obj("chord",labels.get(members.get(i)),"time",rational(time)));
                    }
                    Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
                    String[] parts=beatText.split("/");Object beat=make("math.Rational",Integer.parseInt(parts[0]),parts.length==2?Integer.parseInt(parts[1]):1);
                    Object groups=null,pivot=null;String error=null;boolean fallback=false;List<Object> names=new ArrayList<>();
                    try {List<?> initial=(List<?>)build.invoke(generator,voice,beat);pivot=check.invoke(generator,initial);fallback=pivot!=null&&beatText.equals("1/2");groups=fallback?build.invoke(generator,voice,make("math.Rational",1,4)):initial;for(Object g:(List<?>)groups)names.add(names((Collection<?>)g,labels));}
                    catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();error=cause.getClass().getSimpleName();}
                    scenarios.add(obj("voice",labels.get(voice),"pattern",pattern,"beat",beatText,"times",times,"groups",names,"pivot",labels.get(pivot),"fallback",fallback,"error",error,"before",before,"after",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
                    for(int i=0;i<members.size();i++)call(members.get(i),"setTimeOffset",saved.get(i));
                }
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"declared_extra_group",obj("median",median,"height",4.,"chords",List.of("chord5","chord7")),"scenarios",scenarios));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original private buildGroups/checkGroups on real ordered Voice Chords; explicit beat values, exact boundaries/large gaps/null times, first different real BeamGroup and source HALF-only fallback planning; no automatic force generation claimed","cases",cases));
    }
}
