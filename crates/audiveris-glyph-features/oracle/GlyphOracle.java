// Test adapter. Official numerical and Glyph/RunTable classes are unchanged.
import java.util.*;import java.util.zip.*;import java.io.*;import java.nio.file.*;
import java.awt.*;import java.awt.image.*;
import com.fasterxml.jackson.databind.*;
import org.audiveris.omr.classifier.MixGlyphDescriptor;
import org.audiveris.omr.glyph.Glyph;
import org.audiveris.omr.run.*;
import org.audiveris.omr.math.NeuralNetwork;
public class GlyphOracle {
 public static void main(String[] args)throws Exception{
  ObjectMapper mapper=new ObjectMapper();
  JsonNode old=mapper.readTree(new File(args[3]));
  Map<String,java.util.List<String>> inputs=new LinkedHashMap<>();
  Iterator<Map.Entry<String,JsonNode>> fields=old.fields();while(fields.hasNext()){Map.Entry<String,JsonNode> e=fields.next();java.util.List<String> rows=new ArrayList<>();for(JsonNode r:e.getValue())rows.add(r.asText());inputs.put(e.getKey(),rows);}
  inputs.put("single_pixel",Arrays.asList("#"));inputs.put("single_row",Arrays.asList("#####"));inputs.put("single_column",Arrays.asList("#","#","#","#","#"));
  Font font=Font.createFont(Font.TRUETYPE_FONT,new File(args[2])).deriveFont(48f);
  for(int cp:new int[]{0xe050,0xe260,0xe0a4}){
   if(!font.canDisplay(cp))throw new AssertionError("font glyph missing");
   BufferedImage img=new BufferedImage(200,200,BufferedImage.TYPE_BYTE_GRAY);Graphics2D g=img.createGraphics();g.setColor(Color.WHITE);g.fillRect(0,0,200,200);g.setColor(Color.BLACK);g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING,RenderingHints.VALUE_TEXT_ANTIALIAS_OFF);g.setFont(font);g.drawString(new String(Character.toChars(cp)),70,120);g.dispose();
   int minx=200,miny=200,maxx=-1,maxy=-1;
   for(int y=0;y<200;y++)for(int x=0;x<200;x++)if(img.getRaster().getSample(x,y,0)<128){minx=Math.min(minx,x);miny=Math.min(miny,y);maxx=Math.max(maxx,x);maxy=Math.max(maxy,y);}
   java.util.List<String> rows=new ArrayList<>();for(int y=miny;y<=maxy;y++){StringBuilder row=new StringBuilder();for(int x=minx;x<=maxx;x++)row.append(img.getRaster().getSample(x,y,0)<128?'#':'.');rows.add(row.toString());}inputs.put("bravura_"+Integer.toHexString(cp),rows);
  }
  NeuralNetwork nn;double[] means,stds;
  try(ZipFile z=new ZipFile(args[1])){nn=NeuralNetwork.unmarshal(z.getInputStream(z.getEntry("model.xml")));means=NetworkOracle.vector(z,"means.xml");stds=NetworkOracle.vector(z,"stds.xml");}
  java.util.List<Object> cases=new ArrayList<>();MixGlyphDescriptor desc=new MixGlyphDescriptor();
  for(Map.Entry<String,java.util.List<String>> e:inputs.entrySet())for(int il:new int[]{8,16})for(int offset:new int[]{0,37})for(Orientation orientation:Orientation.values()){
   java.util.List<String> rows=e.getValue();int h=rows.size(),w=rows.get(0).length();RunTable rt=new RunTable(orientation,w,h);java.util.List<Object> runs=new ArrayList<>();
   int seq=orientation==Orientation.HORIZONTAL?h:w,extent=orientation==Orientation.HORIZONTAL?w:h;
   for(int p=0;p<seq;p++){int start=-1;for(int c=0;c<=extent;c++){boolean on=c<extent&&(orientation==Orientation.HORIZONTAL?rows.get(p).charAt(c):rows.get(c).charAt(p))=='#';if(on&&start<0)start=c;if(!on&&start>=0){rt.addRun(p,start,c-start);runs.add(Arrays.asList(p,start,c-start));start=-1;}}}
   Glyph glyph=new Glyph(offset,offset*2,rt);double[] features=desc.getFeatures(glyph,il);
   Map<String,Object> result=NetworkOracle.evaluate(e.getKey()+"_il"+il+"_o"+offset+"_"+orientation,features,nn,means,stds);
   result.put("input",NetworkOracle.obj("bitmap_rows",rows,"width",w,"height",h,"left",offset,"top",offset*2,"interline",il,"orientation",orientation.toString(),"runs",runs,"provenance",e.getKey().startsWith("bravura")?"Bundled Bravura 1.392, embedded OFL-1.1":"newly generated synthetic binary pixels"));cases.add(result);
  }
  mapper.writerWithDefaultPrettyPrinter().writeValue(new File(args[0]),NetworkOracle.obj("source_commit","9e1e55cd2746037d059345881c53e6a6754bffbd","input_labels",desc.getFeatureLabels(),"cases",cases));System.out.println("cases="+cases.size());
 }
}
