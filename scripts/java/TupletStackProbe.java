// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletStackProbe extends BeamRestClearProbe {
    static void setCached(Object object,String name,Object value)throws Exception {for(Class<?> type=object.getClass();type!=null;type=type.getSuperclass())try{java.lang.reflect.Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ex){}throw new NoSuchFieldException(name);}
    static Object containers(List<?> stacks,Object t)throws Exception {
        List<Object> out=new ArrayList<>();for(Object st:stacks){Object m=((List<?>)call(st,"getMeasures")).get(0);out.add(obj("measure",((Collection<?>)call(m,"getTuplets")).contains(t),"stack",((Collection<?>)call(st,"getTuplets")).contains(t),"unassigned",((Collection<?>)field(st,"sheet.rhythm.MeasureStack","stackTuplets")).contains(t)));}return out;
    }
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

            List<Object> scenarios=new ArrayList<>(),rests=new ArrayList<>(),baseline=new ArrayList<>();
            Object generator=make("sheet.rhythm.TupletGenerator",measure),rhythm=make("sheet.rhythm.MeasureRhythm",measure);
            Method remove=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("removeTuplets",Collection.class);remove.setAccessible(true);
            Object familyParam=call(call(sheet,"getStub"),"getMusicFamilyParam");
            call(familyParam,"setSpecific",c("ui.symbol.MusicFamily").getField("Bravura").get(null));
            Object staff=call(system,"getFirstStaff");List<Object> ranges=new ArrayList<>();SortedSet<Integer> positions=new TreeSet<>();
            for(Object st:stacks){Object m=((List<?>)call(st,"getMeasures")).get(0);
                int left=((Number)call(m,"getAbscissa",c("util.HorizontalSide").getField("LEFT").get(null),staff)).intValue();
                int right=((Number)call(m,"getAbscissa",c("util.HorizontalSide").getField("RIGHT").get(null),staff)).intValue();
                ranges.add(obj("left",left,"right",right));
                for(int x:List.of(left-1,left,left+1,(left+right)/2,right-1,right,right+1))positions.add(x);
            }
            for(int x:positions){Object ch=chords.get(0);int y=201,w=11,h=23;Rectangle bounds=new Rectangle(x-w/2,y,w,h);Point head=new Point(x,y+9),tail=new Point(x,y-31);
                setCached(ch,"bounds",bounds);setCached(ch,"headLocation",head);setCached(ch,"tailLocation",tail);
                Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
                List<Object> callbacks=new ArrayList<>();Object listener=java.lang.reflect.Proxy.newProxyInstance(TupletStackProbe.class.getClassLoader(),new Class<?>[]{Class.forName("org.jgrapht.event.GraphListener")},(proxy,method,eventArgs)->{
                    if(method.getName().equals("equals"))return proxy==eventArgs[0];if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);if(method.getName().equals("toString"))return "actual Tuplet Stack boundary observer";
                    Object t=null;String phase=null;
                    if(method.getName().equals("vertexAdded")){t=call(eventArgs[0],"getVertex");phase="vertex_registered";}
                    if(method.getName().equals("edgeAdded")&&c("sig.relation.ChordTupletRelation").isInstance(call(eventArgs[0],"getEdge"))){t=call(eventArgs[0],"getEdgeTarget");phase="chord_tuplet_added";}
                    if(t!=null&&c("sig.inter.TupletInter").isInstance(t))callbacks.add(obj("phase",phase,"sig_attached",call(t,"getSig")!=null,"abnormal",call(t,"isAbnormal"),"containers",containers(stacks,t),"clocks",observe(sig,measure,stack,chords,voices,labels,slots,oldRelations)));
                    return null;});call(sig,"addGraphListener",listener);
                Object t=call(generator,"generateOneTuplet",List.of(ch));call(sig,"removeGraphListener",listener);Rectangle tb=(Rectangle)call(t,"getBounds");
                Object located=call(system,"getStackAt",new Point(tb.x+tb.width/2,tb.y+tb.height/2));
                Object after=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source)),generatedContainers=containers(stacks,t);
                boolean warmed=x%2==0;if(warmed)call(t,"getStaff");remove.invoke(rhythm,List.of(t));
                Object removed=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source)),removedContainers=containers(stacks,t);
                scenarios.add(obj("x",x,"geometry",List.of(obj("chord","chord0","bounds",List.of(bounds.x,bounds.y,w,h),"head",List.of(head.x,head.y),"tail",List.of(tail.x,tail.y))),"located",located==null?null:stacks.indexOf(located),"staff_warmed_before_remove",warmed,"bounds",List.of(tb.x,tb.y,tb.width,tb.height),"before",before,"generated",after,"removed",removed,"callbacks",callbacks,"generated_containers",generatedContainers,"removed_containers",removedContainers,"registered_after_remove",call(sig,"containsVertex",t),"removed_flag",call(t,"isRemoved")));
                // Explicit control cleanup is separate from MeasureRhythm removal.
                for(Object st:stacks)call(st,"removeInter",t);
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"interline",call(call(sheet,"getScale"),"getInterline"),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"ranges",ranges,"scenarios",scenarios));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original actual Stack boundaries/null/other-Stack implicit Tuplet ownership on declared cache geometry of real timed Chords; separate explicit cleanup of other Stack references","cases",cases));
    }
}
