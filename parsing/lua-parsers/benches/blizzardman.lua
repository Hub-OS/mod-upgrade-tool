-- todo: seems to break from spamming stun attacks

local function run_after(character, frame_count, fn)
  local component = Battle.Component.new(character, Lifetimes.Battlestep)

  component.update_func = function()
    frame_count = frame_count - 1

    if frame_count < 0 then
      component:eject()
      fn()
    end
  end

  character:register_component(component)
end

local function wait_for(character, wait_fn, fn)
  local component = Battle.Component.new(character, Lifetimes.Battlestep)

  component.update_func = function()
    if wait_fn() then
      component:eject()
      fn()
    end
  end

  character:register_component(component)
end

local function teleport(blizzardman, tile, endlag, end_callback)
  if blizzardman:get_tile() == tile then
    -- no need to move
    end_callback()
    return
  end

  local anim = blizzardman:get_animation()

  local function teleport_in()
    anim:set_state("MOVE")
    anim:set_playback(Playback.Reverse)
    anim:on_interrupt(end_callback)
    anim:on_complete(function()
      anim:on_interrupt(function() end)
      anim:set_state("IDLE")

      if end_callback then
        local action = Battle.CardAction.new(blizzardman, "IDLE")
        action:set_lockout(make_sequence_lockout())

        local step = Battle.Step.new()
        step.update_func = function(self)
          endlag = endlag - 1

          if endlag <= 0 then
            self:complete_step()
          end
        end
        action:add_step(step)

        action.action_end_func = function()
          end_callback(true)
        end

        blizzardman:card_action_event(action, ActionOrder.Voluntary)
      end
    end)
  end

  anim:set_state("MOVE")
  anim:set_playback(Playback.Once)
  anim:on_interrupt(end_callback)

  anim:on_complete(function()
    if tile ~= nil then
      blizzardman:teleport(tile)
    end

    teleport_in()
  end)
end

local function spawn_snowball_break_artifact(snowball)
  local artifact = Battle.Artifact.new()
  artifact:set_facing(snowball:get_facing())
  artifact:set_texture(snowball:get_texture())

  local anim = artifact:get_animation()
  anim:copy_from(snowball:get_animation())
  anim:set_state("SNOWBALL_BREAKING")
  anim:refresh(artifact:sprite())

  anim:on_complete(function()
    artifact:erase()
  end)

  local offset = snowball:get_offset()
  local tile_offset = snowball:get_tile_offset()
  artifact:set_offset(
    offset.x + tile_offset.x,
    offset.y + tile_offset.y
  )

  snowball:get_field():spawn(artifact, snowball:get_tile())
end

local function spawn_snow_hit_artifact(character)
  local artifact = Battle.Artifact.new()
  artifact:set_facing(Direction.Right)
  artifact:set_texture(Engine.load_texture(_folderpath .. "snow_artifact.png"))

  artifact:set_animation(_folderpath .. "snow_artifact.animation")
  local anim = artifact:get_animation()
  anim:set_state("DEFAULT")
  anim:refresh(artifact:sprite())

  anim:on_complete(function()
    artifact:erase()
  end)

  local char_offset = character:get_offset()
  local char_tile_offset = character:get_tile_offset()
  artifact:set_offset(
    char_offset.x + char_tile_offset.x + (math.random(64) - 32),
    char_offset.y + char_tile_offset.y
  )

  character:get_field():spawn(artifact, character:get_tile())
end

local function find_target(blizzardman)
  local blizzardman_team = blizzardman:get_team()
  local targets = blizzardman:get_field()
      :find_nearest_characters(blizzardman, function(character)
        local team = character:get_team()

        return team ~= blizzardman_team and team ~= Team.Other
      end)

  return targets[1]
end

local function get_random_team_tile(blizzardman)
  local current_tile = blizzardman:get_tile()

  local tiles = blizzardman:get_field()
      :find_tiles(function(tile)
        return blizzardman:can_move_to(tile) and current_tile ~= tile
      end)

  if #tiles == 0 then
    return nil
  end

  return tiles[math.random(#tiles)]
end

local function get_back_tile(blizzardman, y)
  local field = blizzardman:get_field()
  local start_x, end_x, x_step

  if blizzardman:get_facing() == Direction.Left then
    start_x = field:width()
    end_x = 1
    x_step = -1
  else
    start_x = 1
    end_x = field:width()
    x_step = 1
  end

  for x = start_x, end_x, x_step do
    local tile = field:tile_at(x, y)

    if blizzardman:can_move_to(tile) then
      return tile
    end
  end

  return nil
end

local function get_front_tile(blizzardman, y)
  local field = blizzardman:get_field()
  local start_x, end_x, x_step

  if blizzardman:get_facing() == Direction.Left then
    start_x = 1
    end_x = field:width()
    x_step = 1
  else
    start_x = field:width()
    end_x = 1
    x_step = -1
  end

  for x = start_x, end_x, x_step do
    local tile = field:tile_at(x, y)

    if blizzardman:can_move_to(tile) then
      return tile
    end
  end

  return nil
end

local function create_snowball(blizzardman, damage)
  local snowball = Battle.Obstacle.new(blizzardman:get_team())
  snowball:set_facing(blizzardman:get_facing())
  snowball:set_texture(blizzardman:get_texture())
  snowball:set_health(100)
  snowball:set_height(36)
  snowball:share_tile(true)

  local anim = snowball:get_animation()
  anim:copy_from(blizzardman:get_animation())
  anim:set_state("SNOWBALL")
  anim:set_playback(Playback.Loop)

  snowball:set_hit_props(HitProps.new(
    damage,
    Hit.Impact | Hit.Flash | Hit.Flinch,
    Element.Aqua,
    blizzardman:get_context(),
    Drag.None
  ))

  snowball.update_func = function()
    local current_tile = snowball:get_tile()
    current_tile:attack_entities(snowball)

    if not current_tile:is_walkable() then
      snowball:delete()
      return
    end

    if snowball:is_moving() then
      return
    end

    snowball:slide(snowball:get_tile(snowball:get_facing(), 1), frames(10))
  end

  snowball.attack_func = function()
    snowball:delete()
  end

  snowball.delete_func = function()
    spawn_snowball_break_artifact(snowball)
  end

  snowball.can_move_to_func = function()
    return true
  end

  return snowball
end

local function kick_snowball(blizzardman, damage, end_callback)
  local anim = blizzardman:get_animation()
  anim:set_state("KICK")

  anim:on_frame(2, function()
    blizzardman:toggle_counter(true)
  end)

  anim:on_interrupt(function()
    blizzardman:toggle_counter(false)
  end)

  anim:on_frame(3, function()
    blizzardman:toggle_counter(false)
    local snowball = create_snowball(blizzardman, damage)
    local spawn_tile = blizzardman:get_tile(blizzardman:get_facing(), 1)
    blizzardman:get_field():spawn(snowball, spawn_tile)
  end)

  anim:on_complete(function()
    end_callback()
  end)
end

-- kick two snowballs from the top or bottom row to the middle (starting row preferring the same row as the player)
local function snow_rolling(blizzardman, damage, end_callback)
  local target = find_target(blizzardman)

  if not target then
    end_callback()
    return
  end

  local start_row = target:get_tile():y()

  local back_tile = get_back_tile(blizzardman, start_row)

  teleport(blizzardman, back_tile, 25, function()
    kick_snowball(blizzardman, damage, function()
      -- move randomly up/down from the start row
      local y_offset

      if math.random(2) == 1 then
        y_offset = -1
      else
        y_offset = 1
      end

      back_tile = get_back_tile(blizzardman, start_row + y_offset)

      if not back_tile then
        -- try the other way
        back_tile = get_back_tile(blizzardman, start_row - y_offset)
      end

      if back_tile then
        blizzardman:teleport(back_tile)
      end

      kick_snowball(blizzardman, damage, function()
        end_callback()
      end)
    end)
  end)
end

local function create_continuous_hitbox(blizzardman, damage)
  local spell = Battle.Spell.new(blizzardman:get_team())

  spell:set_hit_props(HitProps.new(
    damage,
    Hit.Impact | Hit.Flash | Hit.Flinch,
    Element.Aqua,
    blizzardman:get_context(),
    Drag.None
  ))

  spell.update_func = function()
    spell:get_tile():attack_entities(spell)
  end

  spell.can_move_to_func = function()
    return true
  end

  return spell
end

local function blizzard_breath(blizzardman, damage, end_callback)
  local target = find_target(blizzardman)

  if not target then
    end_callback()
    return
  end

  local front_tile = get_front_tile(blizzardman, target:get_tile():y())
  teleport(blizzardman, front_tile, 0, function(success)
    if not success then
      end_callback()
      return
    end

    local action = Battle.CardAction.new(blizzardman, "BLIZZARD_BREATH")
    local hitboxA = create_continuous_hitbox(blizzardman, damage)
    local hitboxB = create_continuous_hitbox(blizzardman, damage)

    hitboxA.collision_func = function(character)
      spawn_snow_hit_artifact(character)
    end

    hitboxB.collision_func = hitboxA.collision_func

    action.execute_func = function()
      blizzardman:toggle_counter(true)
    end

    action:add_anim_action(2, function()
      Engine.play_audio(blizzardman._breath_sfx, AudioPriority.High)
      blizzardman:toggle_counter(false)

      local facing = blizzardman:get_facing()
      local field = blizzardman:get_field()

      local tile = blizzardman:get_tile(facing, 1)
      field:spawn(hitboxA, tile)
      tile = tile:get_tile(facing, 1)
      field:spawn(hitboxB, tile)
    end)

    action:add_anim_action(15, function()
      hitboxA:erase()
      hitboxB:erase()
    end)

    action.action_end_func = function()
      blizzardman:toggle_counter(false)

      if not hitboxA:is_deleted() then
        hitboxA:erase()
        hitboxB:erase()
      end

      end_callback()
    end

    blizzardman:card_action_event(action, ActionOrder.Voluntary)
  end)
end

local falling_snow_entities = {}

local function erase_falling_snow(snow)
  for i, stored_snow in ipairs(falling_snow_entities) do
    if stored_snow:get_id() == snow:get_id() then
      table.remove(falling_snow_entities, i)
      break
    end
  end

  snow:erase()
end

local function spawn_falling_snow(blizzardman, damage)
  local team = blizzardman:get_team()
  local field = blizzardman:get_field()

  local tiles = field:find_tiles(function(tile)
    if not tile:is_walkable() or tile:get_team() == team then
      return false
    end

    -- avoid spawning where there is already snow
    for _, stored_snow in ipairs(falling_snow_entities) do
      if stored_snow:get_tile() == tile then
        return false
      end
    end

    return true
  end)

  if #tiles == 0 then
    -- no place to spawn
    return
  end

  local tile = tiles[math.random(#tiles)]
  local snow = Battle.Obstacle.new(team)
  snow:set_facing(Direction.Left)
  snow:set_health(1)
  snow:toggle_hitbox(false)
  snow:set_shadow(Shadow.Small)
  snow:show_shadow(true)
  snow:set_texture(blizzardman:get_texture())
  snow:set_height(18)

  local anim = snow:get_animation()
  anim:copy_from(blizzardman:get_animation())
  anim:set_state("FALLING_SNOW")
  anim:refresh(snow:sprite())

  snow:set_hit_props(HitProps.new(
    damage,
    Hit.Impact | Hit.Flash | Hit.Flinch,
    Element.Aqua,
    blizzardman:get_context(),
    Drag.None
  ))

  local elevation = 64
  local hit_something = false
  local melting = false

  local function melt()
    if melting then
      return
    end

    melting = true

    local melting_snow = Battle.Artifact.new()
    melting_snow:set_facing(snow:get_facing())
    melting_snow:set_texture(snow:get_texture())

    local melting_anim = melting_snow:get_animation()
    melting_anim:copy_from(anim)
    melting_anim:set_state("MELTING_SNOW")
    melting_anim:refresh(melting_snow:sprite())

    melting_anim:on_complete(function()
      melting_snow:erase()
    end)

    field:spawn(melting_snow, snow:get_tile())

    erase_falling_snow(snow)
  end

  snow.update_func = function()
    if elevation < 0 then
      snow:toggle_hitbox(true)
      anim:set_state("LANDING_SNOW")
      snow:get_tile():attack_entities(snow)

      anim:on_complete(function()
        if hit_something then
          erase_falling_snow(snow)
        else
          anim:set_state("LANDED_SNOW")
          anim:on_complete(melt)
        end
      end)

      -- no more updating, let the animations handle that
      snow.update_func = function() end
      return
    end

    snow:set_elevation(elevation * 2)
    elevation = elevation - 4
  end

  snow.attack_func = function(character)
    hit_something = true
    spawn_snow_hit_artifact(character)
  end

  snow.delete_func = function()
    melt()
  end

  field:spawn(snow, tile)
  falling_snow_entities[#falling_snow_entities + 1] = snow
end

local function rolling_slider(blizzardman, damage, end_callback)
  local target = find_target(blizzardman)

  if not target then
    end_callback()
    return
  end

  local target_row = target:get_tile():y()
  local end_tile = get_back_tile(blizzardman, target_row)
  teleport(blizzardman, end_tile, 5, function(success)
    if not success then
      end_callback()
      return
    end

    local anim = blizzardman:get_animation()
    local field = blizzardman:get_field()

    local hitbox = create_continuous_hitbox(blizzardman, damage)

    local action = Battle.CardAction.new(blizzardman, "CURLING_UP")
    action:set_lockout(make_sequence_lockout())

    local curling_step = Battle.Step.new()
    local rolling_step = Battle.Step.new()

    action:add_step(curling_step)
    action:add_step(rolling_step)

    action.execute_func = function()
      blizzardman:toggle_counter(true)
      blizzardman:share_tile(true)

      anim:on_complete(function()
        blizzardman:toggle_counter(false)

        anim:set_state("ROLLING")
        anim:set_playback(Playback.Loop)

        field:spawn(hitbox, blizzardman:get_tile())
        curling_step:complete_step()
      end)
    end

    hitbox.update_func = function()
      hitbox:get_tile():attack_entities(hitbox)

      blizzardman:get_tile():remove_entity_by_id(blizzardman:get_id())
      hitbox:get_tile():add_entity(blizzardman)

      local offset = hitbox:get_tile_offset()
      blizzardman:set_offset(offset.x, offset.y)
    end

    rolling_step.update_func = function()
      local current_tile = hitbox:get_tile()

      if not current_tile:is_walkable() then
        if current_tile:is_edge() then
          blizzardman:shake_camera(8, 0.4)
          Engine.play_audio(blizzardman._thud_sfx, AudioPriority.High)

          spawn_falling_snow(blizzardman, damage)

          run_after(blizzardman, math.random(4, 18), function()
            spawn_falling_snow(blizzardman, damage)
          end)
        end

        rolling_step:complete_step()
        return
      end

      if hitbox:is_moving() then
        return
      end

      local dest = hitbox:get_tile(blizzardman:get_facing(), 1)
      hitbox:slide(dest, frames(7))
    end

    action.action_end_func = function()
      hitbox:erase()
      blizzardman:set_offset(0, 0)
      blizzardman:toggle_counter(false)
      blizzardman:share_tile(false)
      end_callback()
    end

    blizzardman:card_action_event(action, ActionOrder.Voluntary)
  end)
end

-- blizzardman's attacks come with a movement plan
-- snow_rolling comes after 3-4 movements
-- blizzard_breath comes after one movement and followed up with rolling_slider
local function pick_plan(blizzardman, plan_number, damage, callback)
  local movements, attack_func

  if plan_number > 1 and math.random(3) == 1 then
    -- 1/3 chance
    movements = 1
    attack_func = function(_blizzardman, _damage, _callback)
      blizzard_breath(_blizzardman, _damage, function()
        rolling_slider(_blizzardman, _damage, _callback)
      end)
    end
  else
    -- 2/3 chance
    movements = math.random(3, 4)
    attack_func = snow_rolling
  end

  local step

  step = function()
    if movements == 0 then
      attack_func(blizzardman, damage, callback)
    else
      movements = movements - 1
      teleport(blizzardman, get_random_team_tile(blizzardman), 60, function()
        wait_for(blizzardman, function()
          return not blizzardman._flinching
        end, step)
      end)
    end
  end

  step()
end

function package_init(blizzardman)
  blizzardman:set_name("BlizMan")
  blizzardman:set_element(Element.Aqua)
  blizzardman:set_height(60)
  blizzardman:set_texture(Engine.load_texture(_folderpath .. "blizzardman.png"))
  blizzardman._breath_sfx = Engine.load_audio(_folderpath .. "wind.ogg")
  blizzardman._thud_sfx = Engine.load_audio(_folderpath .. "thud.ogg")

  local anim = blizzardman:get_animation()
  anim:load(_folderpath .. "blizzardman.animation")
  anim:set_state("IDLE")

  local rank = blizzardman:get_rank()
  local rank_to_hp = {
    [Rank.V1] = 400,
    [Rank.V2] = 1200,
    [Rank.V3] = 1600,
    [Rank.SP] = 2000
  }
  blizzardman:set_health(rank_to_hp[rank])

  local rank_to_damage = {
    [Rank.V1] = 20,
    [Rank.V2] = 40,
    [Rank.V3] = 60,
    [Rank.SP] = 80
  }
  local attack_damage = rank_to_damage[rank]
  local has_plan = false
  local plan_number = 1

  blizzardman.update_func = function()
    if anim:get_state() == "HURT" then
      -- flinching
      return
    end

    if not has_plan then
      pick_plan(blizzardman, plan_number, attack_damage, function()
        has_plan = false
      end)
      has_plan = true
      plan_number = plan_number + 1
    end
  end

  blizzardman:register_status_callback(Hit.Flinch, function()
    anim:set_state("HURT")
    anim:refresh(blizzardman:sprite())

    blizzardman._flinching = true

    anim:on_interrupt(function()
      blizzardman._flinching = false
    end)

    anim:on_complete(function()
      blizzardman._flinching = false
      anim:set_state("IDLE")
    end)
  end)
end
