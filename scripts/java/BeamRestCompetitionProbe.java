// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.*;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Original private lookup with traced real old/manual removals and post-callback fresh addition. */
public class BeamRestCompetitionProbe extends BeamRestPolygonProbe {
    static class TracedRelation extends org.audiveris.omr.sig.relation.BeamRestRelation {
        final java.util.function.Consumer<String> observe;
        TracedRelation(java.util.function.Consumer<String> observe){this.observe=observe;}
        @Override public void removed(org.jgrapht.event.GraphEdgeChangeEvent<org.audiveris.omr.sig.inter.Inter,org.audiveris.omr.sig.relation.Relation> event){observe.accept("remove_entered");super.removed(event);observe.accept("remove_finished");}
    }
    static Map<String,Object> callback(Object sig,Object beam,Object edge,String phase,List<Object> chords,Map<Object,String> labels,Map<Object,Object> slots)throws Exception {
        List<Object> clocks=new ArrayList<>();for(Object ch:chords)clocks.add(obj("label",labels.get(ch),"manual",call(ch,"isManual"),"voice",labels.get(call(ch,"getVoice")),"slot_id",call(ch,"getSlot")==null?null:call(call(ch,"getSlot"),"getId"),"same_baseline_slot",call(ch,"getSlot")==slots.get(ch),"time_offset",rational(call(ch,"getTimeOffset"))));Object portion=call(edge,"getBeamPortion");
        return obj("phase",phase,"edge",labels.get(edge),"beam",labels.get(beam),"registered",call(sig,"containsEdge",edge),"manual",call(edge,"isManual"),"dy",call(edge,"getDistance"),"portion",portion==null?null:portion.toString(),"beam_abnormal",call(beam,"isAbnormal"),"clocks",clocks);
    }
    static List<Object> registrations(Object sig,List<Object> tracked,Map<Object,String> labels)throws Exception {List<Object> rows=new ArrayList<>();for(Object edge:tracked){Object p=call(edge,"getBeamPortion");rows.add(obj("label",labels.get(edge),"registered",call(sig,"containsEdge",edge),"manual",call(edge,"isManual"),"dy",call(edge,"getDistance"),"portion",p==null?null:p.toString()));}return rows;}
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

            List<?> groupChords=(List<?>)call(group,"getChords");Object left=groupChords.get(0),right=groupChords.get(1);
            field(left,"headLocation",new Point(288,220));field(left,"tailLocation",new Point(288,180));field(left,"bounds",new Rectangle(280,200,12,44));field(right,"headLocation",new Point(470,225));field(right,"tailLocation",new Point(470,180));field(right,"bounds",new Rectangle(471,205,13,39));
            Map<String,Object> geometry=obj("input",new int[][]{{288,220},{288,180},{470,225},{470,180}},"left_bounds",List.of(280,200,12,44),"right_bounds",List.of(471,205,13,39));
            List<Object> beams=new ArrayList<>(),declaredBeams=new ArrayList<>();double[] ys={190.5,150.,191.5,190.};for(int i=0;i<ys.length;i++){Object b=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(288,ys[i],470,ys[i]),4.);call(sig,"addVertex",b);if(i==0)call(group,"addMember",b);labels.put(b,"competition_beam"+i);beams.add(b);declaredBeams.add(obj("label",labels.get(b),"kind","BeamInter","median",List.of(288.,ys[i],470.,ys[i]),"height",4.,"in_source_group",i==0));}
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declaredRests=new ArrayList<>();int[][] xy={{356,210},{356,209},{356,240},{356,209},{360,240},{352,240},{292,210},{292,240},{458,240}};for(int[] pt:xy){int x=pt[0],y=pt[1];Object rest=make("sig.inter.RestInter",glyph(x,y),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(measure,"addInter",chord);int i=rests.size();labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);rests.add(chord);chords.add(chord);slots.put(chord,null);declaredRests.add(obj("bounds",List.of(x,y,8,20),"manual",false,"source_measure",true,"beam_link",false));}
            frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));
            List<Map<String,Object>> specs=new ArrayList<>();
            specs.add(obj("name","fresh","candidates",List.of(0),"edges",new int[][]{},"white",List.of(),"slope",0.));
            specs.add(obj("name","manual_loser","candidates",List.of(0),"edges",new int[][]{{0,2,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","parallel_losers","candidates",List.of(0),"edges",new int[][]{{0,2,1},{0,2,0}},"white",List.of(),"slope",0.));
            specs.add(obj("name","reinserted_parallel_losers","candidates",List.of(0),"edges",new int[][]{{0,2,1},{0,2,0}},"white",List.of(),"slope",0.,"reinsert_first",true));
            specs.add(obj("name","white_priority","candidates",List.of(0),"edges",new int[][]{{0,2,1}},"white",List.of(2),"slope",0.));
            specs.add(obj("name","rounded_reject","candidates",List.of(1),"edges",new int[][]{{0,1,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","later_sibling_reject","candidates",List.of(0),"edges",new int[][]{{0,2,1},{0,3,0}},"white",List.of(),"slope",0.));
            specs.add(obj("name","other_beam_loser","candidates",List.of(0),"edges",new int[][]{{1,0,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","sibling_removed_then_other_reject","candidates",List.of(0),"edges",new int[][]{{0,2,1},{2,0,0}},"white",List.of(),"slope",0.));
            specs.add(obj("name","other_equal_rounded","candidates",List.of(0),"edges",new int[][]{{3,0,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","other_removed_then_other_reject","candidates",List.of(0),"edges",new int[][]{{1,0,1},{2,0,0}},"white",List.of(),"slope",0.));
            specs.add(obj("name","sibling_right_contact","candidates",List.of(0),"edges",new int[][]{{0,5,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","sibling_left_contact","candidates",List.of(0),"edges",new int[][]{{0,4,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","skew_enters_right_contact","candidates",List.of(0),"edges",new int[][]{{0,5,1}},"white",List.of(),"slope",.02));
            specs.add(obj("name","skew_leaves_right_contact","candidates",List.of(0),"edges",new int[][]{{0,5,1}},"white",List.of(),"slope",-.03));
            specs.add(obj("name","side_support_callback","candidates",List.of(6),"edges",new int[][]{{0,7,1},{0,8,1}},"white",List.of(),"slope",0.));
            specs.add(obj("name","zero_left_up_head_alias","candidates",List.of(8),"edges",new int[][]{},"white",List.of(),"slope",0.,"geometry",obj("input",new int[][]{{288,220},{288,220},{470,270},{466,300}},"left_bounds",List.of(280,200,12,44),"right_bounds",List.of(471,205,13,39))));
            specs.add(obj("name","candidate_order","candidates",List.of(0,1),"edges",new int[][]{},"white",List.of(),"slope",0.));
            List<Object> scenarios=new ArrayList<>();Method lookup=c("sig.inter.BeamGroupInter").getDeclaredMethod("lookupRests",c("sig.inter.AbstractChordInter"),c("sig.inter.AbstractChordInter"),Set.class,Set.class);lookup.setAccessible(true);
            for(Map<String,Object> spec:specs){Map<String,Object> g=(Map<String,Object>)spec.getOrDefault("geometry",geometry);int[][] ps=(int[][])g.get("input");field(left,"headLocation",new Point(ps[0][0],ps[0][1]));field(left,"tailLocation",new Point(ps[1][0],ps[1][1]));field(right,"headLocation",new Point(ps[2][0],ps[2][1]));field(right,"tailLocation",new Point(ps[3][0],ps[3][1]));spec.put("observed_polygon",points(polygon(group,left,right)));call(sheet,"setSkew",make("sheet.Skew",(Double)spec.get("slope"),sheet));List<Object> tracked=new ArrayList<>(),callbacks=new ArrayList<>();boolean[] recording={false};int[][] edgeSpecs=(int[][])spec.get("edges");for(int i=0;i<edgeSpecs.length;i++){int[] es=edgeSpecs[i];Object b=beams.get(es[0]);Object[] current={null};Object edge=new TracedRelation(phase->{if(recording[0])try{callbacks.add(callback(sig,b,current[0],phase,chords,labels,slots));}catch(Exception e){throw new RuntimeException(e);}});current[0]=edge;labels.put(edge,"old"+i);call(edge,"setManual",es[2]==1);call(sig,"addEdge",b,((List<?>)call(rests.get(es[1]),"getMembers")).get(0),edge);tracked.add(edge);}
                if(Boolean.TRUE.equals(spec.get("reinsert_first"))){Object edge=tracked.get(0);call(sig,"removeEdge",edge);call(sig,"addEdge",beams.get(edgeSpecs[0][0]),((List<?>)call(rests.get(edgeSpecs[0][1]),"getMembers")).get(0),edge);}
                Map<String,Object> before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));List<Object> beforeRelations=registrations(sig,tracked,labels);int[] additions={0};recording[0]=true;
                Object listener=Proxy.newProxyInstance(BeamRestCompetitionProbe.class.getClassLoader(),new Class<?>[]{Class.forName("org.jgrapht.event.GraphListener")},(proxy,method,eventArgs)->{if(method.getName().equals("equals"))return proxy==eventArgs[0];if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);if(method.getName().equals("toString"))return "actual lookup callback observer";if(method.getName().equals("edgeAdded")){Object event=eventArgs[0],edge=call(event,"getEdge");if(c("sig.relation.BeamRestRelation").isInstance(edge)&&!tracked.contains(edge)){labels.put(edge,"added"+additions[0]++);tracked.add(edge);callbacks.add(callback(sig,call(event,"getEdgeSource"),edge,"finished",chords,labels,slots));}}if(method.getName().equals("edgeRemoved")){Object event=eventArgs[0],edge=call(event,"getEdge");if(tracked.contains(edge)&&labels.get(edge).startsWith("added"))callbacks.add(callback(sig,call(event,"getEdgeSource"),edge,"remove_finished",chords,labels,slots));}return null;});call(sig,"addGraphListener",listener);
                LinkedHashSet<Object> candidates=new LinkedHashSet<>(),white=new LinkedHashSet<>();for(int index:(List<Integer>)spec.get("candidates"))candidates.add(rests.get(index));for(int index:(List<Integer>)spec.get("white"))white.add(rests.get(index));lookup.invoke(group,left,right,candidates,white);call(sig,"removeGraphListener",listener);recording[0]=false;
                scenarios.add(obj("spec",spec,"before",before,"before_relations",beforeRelations,"after",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source)),"after_relations",registrations(sig,tracked,labels),"live_order",names(((Collection<?>)call(sig,"edgeSet")).stream().filter(tracked::contains).toList(),labels),"callbacks",callbacks));for(Object edge:tracked)if((Boolean)call(sig,"containsEdge",edge))call(sig,"removeEdge",edge);
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"left",labels.get(left),"right",labels.get(right),"geometry",geometry,"declared_rests",declaredRests,"declared_beams",declaredBeams,"scenarios",scenarios));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","36 original private lookup scenarios on real timed owners; traced original old/manual removed entry/exit; fresh addition observed by later GraphListener after original added callback; white/double-rounded/skew/contact/partial rejection/parallel/candidate encounter; no complete detection or archive","cases",cases));
    }
}
