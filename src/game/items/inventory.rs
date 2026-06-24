use crate::engine::asset_registry::AssetRegistry;
use crate::engine::input_manager::InputManager;
use crate::engine::render::gui::{GuiElement, GuiEngine};
use crate::engine::render::Renderer;
use crate::game::items::item_registry::{ItemID, ItemRegistry};

pub struct Slot {
    pub count: u8,
    pub id: ItemID,
}
impl Slot {
    pub fn get_item_id(&self) -> Option<&ItemID> {
        if self.count > 0{
            return Some(&self.id);
        }
        None
    }
}

const INVENTORY_WIDTH: usize = 6;
const INVENTORY_HEIGHT: usize = 5;
pub struct Inventory{
    slots: Vec<Slot>,
    selected_slot: usize,
}

impl Inventory{
    pub fn new()->Self{
        let mut slots = Vec::with_capacity(INVENTORY_WIDTH*INVENTORY_HEIGHT);
        for x in 0..INVENTORY_WIDTH{
            for y in 0..INVENTORY_HEIGHT{
                slots.push(Slot{
                    count: 0,
                    id: "".to_string(),
                })
            }
        }
        Self{
            slots,
            selected_slot: 1,
        }
    }
    pub fn draw_hotbar(&self,gui: &mut GuiEngine,asset_registry: &AssetRegistry,item_registry: &ItemRegistry){
        for s in 0..INVENTORY_WIDTH{
            if let Some(item) = item_registry.definitions.get(&self.slots[s].id){
                gui.add_gui_element(GuiElement{
                    material: asset_registry.item_icon_mat,
                    texture_index: item.icon_index,
                    position: [-0.9 + s as f32 * 0.11,0.9,0.0],
                    scale: 0.05,
                });
            }
            gui.add_gui_element(GuiElement{
                material: asset_registry.gui_mat,
                texture_index: 0,
                position: [-0.9 + s as f32 * 0.11,0.9,0.1],
                scale: 0.1,
            });
        }
    }
    pub fn add_item(&mut self,id: ItemID,count:u8,slot:usize){
        self.slots[slot].count = count;
        self.slots[slot].id = id;
    }
    pub fn held_item(&self) -> Option<&ItemID>{
        self.slots[self.selected_slot].get_item_id()
    }
    pub fn handle_input(&mut self,input_manager: &InputManager){
        for (i,num) in input_manager.nums.iter().enumerate(){
            if *num{
                self.selected_slot = i.saturating_sub(1);
            }
        }
    }
}