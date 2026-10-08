// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.*;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

public class BeamRestDetectProbe extends BeamRestCompetitionProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) for(String scenario:List.of("white_plain","black_wins","strict_boundary","null_nearest","captured_rest")) {
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
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declaredRests=new ArrayList<>();int[][] xy={{356,210},{356,209},{390,210},{284,210},{466,210},{900,210},{396,210}};for(int[] pt:xy){int x=pt[0],y=pt[1];Object rest=make("sig.inter.RestInter",glyph(x,y),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(measure,"addInter",chord);int i=rests.size();labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);rests.add(chord);chords.add(chord);slots.put(chord,null);declaredRests.add(obj("bounds",List.of(x,y,8,20),"manual",false,"source_measure",true,"beam_link",false));}

            List<Object> links=new ArrayList<>();
            int[] whites=scenario.equals("strict_boundary")?new int[]{3,4}:scenario.equals("null_nearest")?new int[]{5}:new int[]{0};
            for(int i:whites){Object edge=make("sig.relation.NextInVoiceRelation");call(edge,"setManual",true);call(sig,"addEdge",rests.get(i),left,edge);links.add(obj("source","rest_chord"+i,"target",labels.get(left),"kind","NextInVoiceRelation","manual",true));}
            if(scenario.equals("white_plain")){Object edge=make("sig.relation.NextInVoiceRelation");call(sig,"addEdge",rests.get(0),rests.get(2),edge);links.add(obj("source","rest_chord0","target","rest_chord2","kind","NextInVoiceRelation","manual",false));}
            if(scenario.equals("black_wins")){Object edge=make("sig.relation.SeparateVoiceRelation");call(sig,"addEdge",left,rests.get(0),edge);links.add(obj("source",labels.get(left),"target","rest_chord0","kind","SeparateVoiceRelation","manual",false));}
            if(scenario.equals("null_nearest")){field(right,"headLocation",new Point(1000,225));field(right,"tailLocation",new Point(1000,180));field(right,"bounds",new Rectangle(1001,205,13,39));geometry=obj("input",new int[][]{{288,220},{288,180},{1000,225},{1000,180}},"left_bounds",List.of(280,200,12,44),"right_bounds",List.of(1001,205,13,39));}
            List<Object> tracked=new ArrayList<>();
            if(scenario.equals("captured_rest")){field(left,"headLocation",new Point(288,220));field(left,"tailLocation",new Point(288,220));field(right,"headLocation",new Point(470,220));field(right,"tailLocation",new Point(470,220));geometry=obj("input",new int[][]{{288,220},{288,220},{470,220},{470,220}},"left_bounds",List.of(280,200,12,44),"right_bounds",List.of(471,205,13,39));Object edge=make("sig.relation.BeamRestRelation");call(edge,"setManual",true);call(sig,"addEdge",beams.get(0),((List<?>)call(rests.get(6),"getMembers")).get(0),edge);labels.put(edge,"old0");tracked.add(edge);}
            List<?> captured=(List<?>)call(group,"getChords");List<Object> black=new ArrayList<>((Collection<?>)BeamRestLinkedProbe.linked(group,captured,false,labels,"SeparateVoiceRelation")),white=new ArrayList<>((Collection<?>)BeamRestLinkedProbe.linked(group,captured,true,labels,"NextInVoiceRelation"));white.removeAll(black);
            List<Object> plain=names((Collection<?>)call(measure,"getRestChords"),labels);plain.removeAll(black);plain.removeAll(white);
            Map<String,Object> candidate=obj("seeds",names(captured,labels),"black",black,"white",white,"plain",plain);
            Map<String,Object> before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));List<Object> beforeRelations=registrations(sig,tracked,labels),callbacks=new ArrayList<>();int[] additions={0};
            Object listener=Proxy.newProxyInstance(BeamRestDetectProbe.class.getClassLoader(),new Class<?>[]{Class.forName("org.jgrapht.event.GraphListener")},(proxy,method,eventArgs)->{if(method.getName().equals("equals"))return proxy==eventArgs[0];if(method.getName().equals("hashCode"))return System.identityHashCode(proxy);if(method.getName().equals("toString"))return "actual full group detection observer";if(method.getName().equals("edgeAdded")){Object event=eventArgs[0],edge=call(event,"getEdge");if(c("sig.relation.BeamRestRelation").isInstance(edge)&&!tracked.contains(edge)){labels.put(edge,"added"+additions[0]++);tracked.add(edge);callbacks.add(callback(sig,call(event,"getEdgeSource"),edge,"finished",chords,labels,slots));}}if(method.getName().equals("edgeRemoved")){Object event=eventArgs[0],edge=call(event,"getEdge");if(tracked.contains(edge))callbacks.add(callback(sig,call(event,"getEdgeSource"),edge,"remove_finished",chords,labels,slots));}return null;});call(sig,"addGraphListener",listener);
            call(group,"detectInterleavedRests");call(sig,"removeGraphListener",listener);
            cases.add(obj("name",mode+"_"+scenario,"declared_last_offset",offset.toString(),"left",labels.get(left),"right",labels.get(right),"geometry",geometry,"declared_rests",declaredRests,"declared_beams",declaredBeams,"declared_links",links,"preexisting_rest",scenario.equals("captured_rest")?6:null,"candidates",candidate,"before",before,"before_relations",beforeRelations,"after",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source)),"after_relations",registrations(sig,tracked,labels),"callbacks",callbacks));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original complete group detect on real timed owners; held seed RestChord; white incoming/transitive before plain, Separate blacklist, strict endpoints, null nearest, post-callback observations; no full MeasureRhythm preparation or archive","cases",cases));
    }
}
