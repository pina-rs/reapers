# Reading the environment

The aim: a husk should live in the room you are standing in. One that walks should run across your floor and round your furniture, one that flies should stay in your air, and it should only show itself once the phone has worked out where it can go.

This page says what the game does about that today, what it cannot do yet, and the ways to close the gap. It ends with the decisions that are yours.

## What is built

The flow you asked for is in the game on every phone:

1. A hunt opens with "Something is nearby".
2. You look around. For a walker you sweep the phone across the floor; for a flier you look around at head height. A dial fills in as the phone reads each slice of the room.
3. Once an unbroken arc of 115° has been read, the husk gathers into view inside it. It never leaves that arc.
4. A walker stands on the floor, with a shadow under it, and runs across the floor when it bolts. A flier holds the air.

What is doing the reading is modest:

- **Which way is down** comes from the accelerometer.
- **Where the floor is** is taken to be 1.45 m below the phone along that line: a standing adult holding a phone at chest height.
- **Where a husk may go** is wherever you have looked steadily, at the right height.

That is enough for a husk to sit convincingly on an open floor. It is not understanding the room.

## What it cannot do yet

- It does not know what is on the floor. A husk will walk through a sofa and stand inside a table.
- It does not know how high the phone really is. Sit down and walkers float.
- It does not follow you moving, only turning. Take a step and every husk comes with you.
- A husk is never hidden behind anything real.

## Your Seeker has no ARCore

This matters, because ARCore is how an Android phone normally answers all four of those.

- Google Play Services for AR is not installed on the Seeker, and the phone does not report the `android.hardware.camera.ar` feature that certified devices do. I checked both over `adb` on 1 October 2026.
- No Solana Mobile device is on [Google's list of ARCore supported devices](https://developers.google.com/ar/devices). The list is what decides whether the service can be installed at all.

So on the Seeker the platform will not give us surfaces. Anything smarter than the floor-from-gravity above has to come from the camera picture itself.

## The ways forward

### 1. ARKit and ARCore, where the phone has them

On an iPhone, and on any Android phone on Google's list, the system tracks the phone's position as well as its direction, finds floors, walls and table tops, and (on phones with a depth sensor) knows how far away everything is.

- **Gives:** real floor height, real surfaces to stand on, husks that stay put when you walk, husks hidden behind furniture.
- **Costs:** a native bridge on each platform that hands the game the camera picture, the phone's position and the surfaces it has found. The game already takes the phone's direction through one interface (`PoseSource`), and where a husk may go through another (`RoomReader` produces a `HuntPlan`), so this slots in behind those two. The hunt's rules then need a husk's route to follow a surface's outline instead of an arc.
- **Does not help:** the Seeker.

### 2. A depth model on the phone

A small neural network can estimate, from one ordinary camera frame, how far away each part of the picture is. Run on the frames taken during the scan, and combined with knowing which way is down, that gives a rough map of the floor around you with the furniture marked on it.

- **Gives:** on the Seeker and every other phone, a husk that goes round the sofa instead of through it, a better guess at the floor's real height, and later, husks hidden behind things.
- **Costs:** bundling a model of roughly 5 to 25 MB, running it on the phone's graphics chip, and tuning it on real rooms. Depth from one camera is approximate: it is good at "there is something in the way here" and poor at exact distances. How fast it runs on the Seeker's chip has to be measured, not assumed.
- **Stays on the phone.** No pictures leave it.

### 3. A vision model on the server

Send a handful of frames from the scan to our server, along with which way each was pointing. A vision model looks at them and sends back a description of the room: where the open floor is, what is in the way and roughly how far, and what a creature could stand on or hide behind.

- **Gives:** the richest understanding of the three. It knows a sofa is a sofa, so a husk can duck behind it or jump onto a table. It works on any phone.
- **Costs:** a few seconds of waiting, which the scan can cover; money for every hunt; a connection. And it means sending pictures of the inside of people's homes to a server, which needs their clear consent and a firm rule about not keeping them.
- **Needs:** an API key for the model. I have not set one up, and would not without being asked.

## What I would do

1. Keep what is built as the floor every phone gets.
2. Build option 2 next. It is the only one that improves the Seeker, it keeps pictures on the phone, and it feeds the same `HuntPlan` the hunt already plays in.
3. Add option 1 for iPhones after that. It is the best result where it is available, and iPhones are most of the market for an AR game.
4. Treat option 3 as an extra for later, for the "hides behind the sofa" behaviour, once there is a consent screen and a privacy rule to go with it.

## Decisions that are yours

- **May I add a depth model to the app?** It means downloading model weights from a public model hub and shipping them inside the build. I have not downloaded anything.
- **Is the iPhone a target now, or is this Seeker-first?** That decides whether option 1 or option 2 comes first.
- **Are you willing to send camera frames to a server at all?** If not, option 3 is off the table and nothing else changes.
- **Is 1.45 m the right guess, or should the game ask?** A one-time "hold the phone as you would to play" step, or a sitting and standing switch, would fix the commonest way the floor is wrong today.
